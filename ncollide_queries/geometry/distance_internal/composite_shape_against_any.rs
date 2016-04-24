use std::marker::PhantomData;
use na::{Identity, Translate, Translation};
use na;
use entities::bounding_volume::{self, HasBoundingVolume, AABB};
use entities::partitioning::BVTCostFn;
use entities::shape::CompositeShape;
use entities::inspection::Shape;
use point::PointQuery;
use geometry::distance_internal;
use math::{Point, Vector, Isometry};

/// Smallest distance between a composite shape and any other shape.
pub fn composite_shape_against_any<P, M, G1: ?Sized, G2: ?Sized>(m1: &M, g1: &G1, m2: &M, g2: &G2) -> <P::Vect as Vector>::Scalar
    where P:  Point,
          P::Vect: Translate<P>,
          M:  Isometry<P, P::Vect> + Translation<P::Vect>,
          G1: CompositeShape<P, M>,
          G2: Shape<P, M> + HasBoundingVolume<M, AABB<P>> {
    let mut cost_fn = CompositeShapeAgainstAnyDistCostFn::new(m1, g1, m2, g2);

    g1.bvt().best_first_search(&mut cost_fn).map(|(_, res)| res).expect("The composite shape must not be empty.")
}

/// Smallest distance between a shape and a composite shape.
pub fn any_against_composite_shape<P, M, G1: ?Sized, G2: ?Sized>(m1: &M, g1: &G1, m2: &M, g2: &G2) -> <P::Vect as Vector>::Scalar
    where P:  Point,
          P::Vect: Translate<P>,
          M:  Isometry<P, P::Vect> + Translation<P::Vect>,
          G1: Shape<P, M> + HasBoundingVolume<M, AABB<P>>,
          G2: CompositeShape<P, M> {
    composite_shape_against_any(m2, g2, m1, g1)
}

struct CompositeShapeAgainstAnyDistCostFn<'a, P: 'a + Point, M: 'a, G1: ?Sized + 'a, G2: ?Sized + 'a> {
    msum_shift:  P::Vect,
    msum_margin: P::Vect,

    m1: &'a M,
    g1: &'a G1,
    m2: &'a M,
    g2: &'a G2,

    point_type: PhantomData<P>
}

impl<'a, P, M, G1: ?Sized, G2: ?Sized> CompositeShapeAgainstAnyDistCostFn<'a, P, M, G1, G2>
    where P:  Point,
          M:  Isometry<P, P::Vect>,
          G1: CompositeShape<P, M>,
          G2: Shape<P, M> + HasBoundingVolume<M, AABB<P>> {
    pub fn new(m1: &'a M, g1: &'a G1, m2: &'a M, g2: &'a G2)
        -> CompositeShapeAgainstAnyDistCostFn<'a, P, M, G1, G2> {

        let ls_m2 = na::inverse(m1).expect("The transformation `m1` must be inversible.") * *m2;
        let ls_aabb2 = bounding_volume::aabb(g2, &ls_m2);

        CompositeShapeAgainstAnyDistCostFn {
            msum_shift:  -ls_aabb2.center().to_vector(),
            msum_margin: ls_aabb2.half_extents(),
            m1:          m1,
            g1:          g1,
            m2:          m2,
            g2:          g2,
            point_type:  PhantomData
        }
    }
}

impl<'a, P, M, G1: ?Sized, G2: ?Sized>
BVTCostFn<<P::Vect as Vector>::Scalar, usize, AABB<P>>
for CompositeShapeAgainstAnyDistCostFn<'a, P, M, G1, G2>
    where P:  Point,
          P::Vect: Translate<P>,
          M:  Isometry<P, P::Vect> + Translation<P::Vect>,
          G1: CompositeShape<P, M>,
          G2: Shape<P, M> + HasBoundingVolume<M, AABB<P>> {
    type UserData = <P::Vect as Vector>::Scalar;
    #[inline]
    fn compute_bv_cost(&mut self, bv: &AABB<P>) -> Option<<P::Vect as Vector>::Scalar> {
        // Compute the minkowski sum of the two AABBs.
        let msum = AABB::new(*bv.mins() + self.msum_shift + (-self.msum_margin),
                             *bv.maxs() + self.msum_shift + self.msum_margin);

        // Compute the distance to the origin.
        Some(msum.distance_to_point(&Identity::new(), &na::origin(), true))
    }

    #[inline]
    fn compute_b_cost(&mut self, b: &usize) -> Option<(<P::Vect as Vector>::Scalar, <P::Vect as Vector>::Scalar)> {
        let mut res = None;

        self.g1.map_transformed_part_at(*b, self.m1, &mut |m1, g1| {
            let distance = distance_internal::distance(m1, g1, self.m2, self.g2);

            res = Some((distance, distance))
        });

        res
    }
}
