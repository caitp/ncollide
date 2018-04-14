//! Two-dimensional penetration depth queries using the Expanding Polytope Algorithm.

use std::marker::PhantomData;
use std::collections::BinaryHeap;
use std::cmp::Ordering;
use num::Bounded;
use approx::ApproxEq;

use alga::general::{Id, Real};
use na::{self, Unit};

use utils;
use shape::{AnnotatedMinkowskiSum, AnnotatedPoint, Reflection, SupportMap};
use query::algorithms::gjk;
use query::algorithms::simplex::Simplex;
use math::Point;

#[derive(Copy, Clone, PartialEq)]
struct FaceId<N: Real> {
    id: usize,
    neg_dist: N,
}

impl<N: Real> FaceId<N> {
    fn new(id: usize, neg_dist: N) -> Option<Self> {
        if neg_dist > gjk::eps_tol() {
            println!(
                "EPA: the origin was outside of the CSO: {} > tolerence ({})",
                neg_dist,
                gjk::eps_tol::<N>()
            );
            None
        } else {
            Some(FaceId { id, neg_dist })
        }
    }
}

impl<N: Real> Eq for FaceId<N> {}

impl<N: Real> PartialOrd for FaceId<N> {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.neg_dist.partial_cmp(&other.neg_dist)
    }
}

impl<N: Real> Ord for FaceId<N> {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        if self.neg_dist < other.neg_dist {
            Ordering::Less
        } else if self.neg_dist > other.neg_dist {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

#[derive(Clone, Debug)]
struct Face<N: Real> {
    pts: [usize; 2],
    normal: Unit<Vector<N>>,
    proj: Point<N>,
    deleted: bool,
    _marker: PhantomData<P>,
}

impl<N: Real> Face<P> {
    pub fn new(vertices: &[Point<N>], pts: [usize; 2]) -> (Self, bool) {
        if let Some(proj) = project_origin(&vertices[pts[0]], &vertices[pts[1]]) {
            (Self::new_with_proj(vertices, proj, pts), true)
        } else {
            (Self::new_with_proj(vertices, Point::origin(), pts), false)
        }
    }

    pub fn new_with_proj(vertices: &[Point<N>], proj: Point<N>, pts: [usize; 2]) -> Self {
        let normal;
        let deleted;

        if let Some(n) = Point::ccw_face_normal(&[&vertices[pts[0]], &vertices[pts[1]]]) {
            normal = n;
            deleted = false;
        } else {
            normal = Unit::new_unchecked(na::zero());
            deleted = true;
        }

        let _marker = PhantomData;

        Face {
            pts,
            normal,
            proj,
            deleted,
            _marker,
        }
    }
}

/// The Expanding Polytope Algorithm in 2D.
pub struct EPA2<N: Real> {
    vertices: Vec<Point<N>>,
    faces: Vec<Face<P>>,
    heap: BinaryHeap<FaceId<N>>,
}

impl<N: Real> EPA2<P> {
    /// Creates a new instance of the 2D Expanding Polytope Algorithm.
    pub fn new() -> Self {
        EPA2 {
            vertices: Vec::new(),
            faces: Vec::new(),
            heap: BinaryHeap::new(),
        }
    }

    fn reset(&mut self) {
        self.vertices.clear();
        self.faces.clear();
        self.heap.clear();
    }

    /// Projects the origin on a shape unsing the EPA algorithm.
    ///
    /// The origin is assumed to be located inside of the shape.
    /// Returns `None` if the EPA fails to converge or if `g1` and `g2` are not penetrating.
    pub fn project_origin<M, S, G: ?Sized>(
        &mut self,
        m: &Isometry<N>,
        shape: &G,
        simplex: &S,
    ) -> Option<(P, Unit<Vector<N>>)>
    where
        S: Simplex<P>,
        G: SupportMap<N>,
    {
        let _eps = N::default_epsilon();
        let _eps_tol = _eps * na::convert(100.0f64);

        self.reset();

        /*
         * Initialization.
         */
        for i in 0..simplex.dimension() + 1 {
            self.vertices.push(simplex.point(i));
        }

        if simplex.dimension() == 0 {
            let mut n: Vector<N> = na::zero();
            n[1] = na::one();
            return Some((Point::origin(), Unit::new_unchecked(n)));
        } else if simplex.dimension() == 2 {
            let dp1 = self.vertices[1] - self.vertices[0];
            let dp2 = self.vertices[2] - self.vertices[0];

            if utils::perp2(&dp1, &dp2) < na::zero() {
                self.vertices.swap(1, 2)
            }

            let pts1 = [0, 1];
            let pts2 = [1, 2];
            let pts3 = [2, 0];

            let (face1, proj_is_inside1) = Face::new(&self.vertices, pts1);
            let (face2, proj_is_inside2) = Face::new(&self.vertices, pts2);
            let (face3, proj_is_inside3) = Face::new(&self.vertices, pts3);

            self.faces.push(face1);
            self.faces.push(face2);
            self.faces.push(face3);

            if proj_is_inside1 {
                let dist1 = na::dot(
                    self.faces[0].normal.as_ref(),
                    &self.vertices[0].coords,
                );
                self.heap.push(FaceId::new(0, -dist1)?);
            }

            if proj_is_inside2 {
                let dist2 = na::dot(
                    self.faces[1].normal.as_ref(),
                    &self.vertices[1].coords,
                );
                self.heap.push(FaceId::new(1, -dist2)?);
            }

            if proj_is_inside3 {
                let dist3 = na::dot(
                    self.faces[2].normal.as_ref(),
                    &self.vertices[2].coords,
                );
                self.heap.push(FaceId::new(2, -dist3)?);
            }
        } else {
            let pts1 = [0, 1];
            let pts2 = [1, 0];

            self.faces
                .push(Face::new_with_proj(&self.vertices, Point::origin(), pts1));
            self.faces
                .push(Face::new_with_proj(&self.vertices, Point::origin(), pts2));

            let dist1 = na::dot(
                self.faces[0].normal.as_ref(),
                &self.vertices[0].coords,
            );
            let dist2 = na::dot(
                self.faces[1].normal.as_ref(),
                &self.vertices[1].coords,
            );

            self.heap.push(FaceId::new(0, dist1)?);
            self.heap.push(FaceId::new(1, dist2)?);
        }

        let mut niter = 0;
        let mut max_dist = N::max_value();
        let mut best_face_id = *self.heap.peek().unwrap();

        /*
         * Run the expansion.
         */
        while let Some(face_id) = self.heap.pop() {
            // Create new faces.
            let face = self.faces[face_id.id].clone();

            if face.deleted {
                continue;
            }

            let support_point = shape.support_point(m, &face.normal);
            let support_point_id = self.vertices.len();
            self.vertices.push(support_point);

            let candidate_max_dist = na::dot(&support_point.coords, &face.normal);

            if candidate_max_dist < max_dist {
                best_face_id = face_id;
                max_dist = candidate_max_dist;
            }

            let curr_dist = -face_id.neg_dist;

            if max_dist - curr_dist < _eps_tol {
                let best_face = &self.faces[best_face_id.id];
                return Some((best_face.proj, best_face.normal));
            }

            let pts1 = [face.pts[0], support_point_id];
            let pts2 = [support_point_id, face.pts[1]];

            let new_faces = [
                Face::new(&self.vertices, pts1),
                Face::new(&self.vertices, pts2),
            ];

            for f in new_faces.into_iter() {
                if f.1 {
                    let dist = na::dot(f.0.normal.as_ref(), &f.0.proj.coords);
                    if dist < curr_dist {
                        // FIXME: if we reach this point, there were issues due to
                        // numerical errors.
                        return Some((f.0.proj, f.0.normal));
                    }

                    if !f.0.deleted {
                        self.heap.push(FaceId::new(self.faces.len(), -dist)?);
                    }
                }

                self.faces.push(f.0.clone());
            }

            niter += 1;
            if niter > 10000 {
                println!("EPA did not converge after 1000 iterations… stopping the iterations.");
                return None;
            }
        }

        let best_face = &self.faces[best_face_id.id];
        return Some((best_face.proj, best_face.normal));
    }
}

/// Computes the pair of closest points at the extremities of the minimal translational vector between `g1` and `g2`.
///
/// Returns `None` if the EPA fails to converge or if `g1` and `g2` are not penetrating.
pub fn closest_points<P, M, S, G1: ?Sized, G2: ?Sized>(
    epa: &mut EPA2<AnnotatedPoint<P>>,
    m1: &Isometry<N>,
    g1: &G1,
    m2: &Isometry<N>,
    g2: &G2,
    simplex: &S,
) -> Option<(Point<N>, Point<N>, Unit<Vector<N>>)>
where
    N: Real,
    S: Simplex<AnnotatedPoint<P>>,
    G1: SupportMap<N>,
    G2: SupportMap<N>,
{
    let reflect2 = Reflection::new(g2);
    let cso = AnnotatedMinkowskiSum::new(m1, g1, m2, &reflect2);

    let (p, n) = epa.project_origin(&Id::new(), &cso, simplex)?;
    Some((*p.orig1(), -*p.orig2(), n))
}

fn project_origin<N: Real>(a: &P, b: &P) -> Option<P> {
    let ab = *b - *a;
    let ap = -a.coords;
    let ab_ap = na::dot(&ab, &ap);
    let sqnab = na::norm_squared(&ab);

    if sqnab == na::zero() {
        return None;
    }

    let position_on_segment;

    let _eps: N = gjk::eps_tol();

    if ab_ap < -_eps || ab_ap > sqnab + _eps {
        // Voronoï region of vertex 'a' or 'b'.
        None
    } else {
        // Voronoï region of the segment interior.
        position_on_segment = ab_ap / sqnab;

        let mut res = *a;
        let _1 = na::one::<N>();
        res.axpy(position_on_segment, b, _1 - position_on_segment);

        Some(res)
    }
}