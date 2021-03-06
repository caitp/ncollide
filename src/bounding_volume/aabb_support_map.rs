use na::Real;
use bounding_volume::{HasBoundingVolume, AABB};
use bounding_volume;
#[cfg(feature = "dim3")]
use shape::{Capsule, Cone, Cylinder, Triangle};
use shape::Segment;
use math::Isometry;

#[cfg(feature = "dim3")]
impl<N: Real> HasBoundingVolume<N, AABB<N>> for Cone<N> {
    #[inline]
    fn bounding_volume(&self, m: &Isometry<N>) -> AABB<N> {
        bounding_volume::support_map_aabb(m, self)
    }
}

#[cfg(feature = "dim3")]
impl<N: Real> HasBoundingVolume<N, AABB<N>> for Cylinder<N> {
    #[inline]
    fn bounding_volume(&self, m: &Isometry<N>) -> AABB<N> {
        bounding_volume::support_map_aabb(m, self)
    }
}

#[cfg(feature = "dim3")]
impl<N: Real> HasBoundingVolume<N, AABB<N>> for Capsule<N> {
    #[inline]
    fn bounding_volume(&self, m: &Isometry<N>) -> AABB<N> {
        bounding_volume::support_map_aabb(m, self)
    }
}

#[cfg(feature = "dim3")]
impl<N: Real> HasBoundingVolume<N, AABB<N>> for Triangle<N> {
    #[inline]
    fn bounding_volume(&self, m: &Isometry<N>) -> AABB<N> {
        // FIXME: optimize that
        bounding_volume::support_map_aabb(m, self)
    }
}

impl<N: Real> HasBoundingVolume<N, AABB<N>> for Segment<N> {
    #[inline]
    fn bounding_volume(&self, m: &Isometry<N>) -> AABB<N> {
        // FIXME: optimize that
        bounding_volume::support_map_aabb(m, self)
    }
}
