use std::marker::PhantomData;
use na::Translate;
use na;
use math::{Point, Vector, Isometry};
use entities::shape::Ball;
use entities::inspection::Shape;
use queries::geometry::Proximity;
use queries::geometry::proximity_internal;
use narrow_phase::{ProximityDetector, ProximityDispatcher};


/// Proximity detector between two balls.
pub struct BallBallProximityDetector<P: Point, M> {
    proximity: Proximity,
    pt_type:   PhantomData<P>, // FIXME: can we avoid this?
    mat_type:  PhantomData<M>  // FIXME: can we avoid this?
}

impl<P: Point, M> Clone for BallBallProximityDetector<P, M> {
    fn clone(&self) -> BallBallProximityDetector<P, M> {
        BallBallProximityDetector {
            proximity: self.proximity,
            pt_type:   PhantomData,
            mat_type:  PhantomData
        }
    }
}

impl<P: Point, M> BallBallProximityDetector<P, M> {
    /// Creates a new persistent collision detector between two balls.
    #[inline]
    pub fn new() -> BallBallProximityDetector<P, M> {
        BallBallProximityDetector {
            proximity: Proximity::Disjoint,
            pt_type:   PhantomData,
            mat_type:  PhantomData
        }
    }
}

impl<P, M> ProximityDetector<P, M> for BallBallProximityDetector<P, M>
    where P: Point,
          M: Isometry<P, P::Vect> {
    fn update(&mut self, _: &ProximityDispatcher<P, M>,
              ma: &M, a: &Shape<P, M>,
              mb: &M, b: &Shape<P, M>,
              margin: <P::Vect as Vector>::Scalar)
              -> bool {
        let ra = a.desc();
        let rb = b.desc();

        if let (Some(a), Some(b)) = (ra.as_shape::<Ball<<P::Vect as Vector>::Scalar>>(),
                                     rb.as_shape::<Ball<<P::Vect as Vector>::Scalar>>()) {
            self.proximity = proximity_internal::ball_against_ball(
                &ma.translate(&na::origin()),
                a,
                &mb.translate(&na::origin()),
                b,
                margin);

            true
        }
        else {
            false
        }
    }

    #[inline]
    fn proximity(&self) -> Proximity {
        self.proximity
    }
}
