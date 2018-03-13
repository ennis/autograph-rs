use nalgebra::*;
use std::cmp::Ord;
use num_traits::Bounded;

#[derive(Copy, Clone, Debug)]
pub struct AABB<N: Real> {
    pub min: Point3<N>,
    pub max: Point3<N>,
}

pub fn partial_min<T: PartialOrd>(a: T, b: T) -> T {
    if a < b {
        a
    } else {
        b
    }
}

pub fn partial_max<T: PartialOrd>(a: T, b: T) -> T {
    if a < b {
        b
    } else {
        a
    }
}

pub fn cw_min3<N: Scalar + PartialOrd>(a: &Vector3<N>, b: &Vector3<N>) -> Vector3<N> {
    Vector3::new(
        partial_min(a.x, b.x),
        partial_min(a.y, b.y),
        partial_min(a.z, b.z),
    )
}

pub fn cw_max3<N: Scalar + PartialOrd>(a: &Vector3<N>, b: &Vector3<N>) -> Vector3<N> {
    Vector3::new(
        partial_max(a.x, b.x),
        partial_max(a.y, b.y),
        partial_max(a.z, b.z),
    )
}

pub fn cw_min4<N: Scalar + PartialOrd>(a: &Vector4<N>, b: &Vector4<N>) -> Vector4<N> {
    Vector4::new(
        partial_min(a.x, b.x),
        partial_min(a.y, b.y),
        partial_min(a.z, b.z),
        partial_min(a.w, b.w),
    )
}

pub fn cw_max4<N: Scalar + PartialOrd>(a: &Vector4<N>, b: &Vector4<N>) -> Vector4<N> {
    Vector4::new(
        partial_max(a.x, b.x),
        partial_max(a.y, b.y),
        partial_max(a.z, b.z),
        partial_max(a.w, b.w),
    )
}

impl<N: Real> AABB<N> {
    pub fn size(&self) -> Vector3<N> {
        self.max - self.min
    }

    /// Reference:
    /// http://dev.theomader.com/transform-bounding-boxes/
    pub fn transform(&self, tr: &Affine3<N>) -> AABB<N> {
        let trh = tr.matrix();
        let xa = trh.column(0) * self.min[0];
        let xb = trh.column(0) * self.max[0];
        let ya = trh.column(1) * self.min[1];
        let yb = trh.column(1) * self.max[1];
        let za = trh.column(2) * self.min[2];
        let zb = trh.column(2) * self.max[2];

        let min = cw_min4(&xa, &xb) + cw_min4(&ya, &yb) + cw_min4(&za, &zb) + trh.column(3);
        let max = cw_max4(&xa, &xb) + cw_max4(&ya, &yb) + cw_max4(&za, &zb) + trh.column(3);

        AABB {
            min: Point3::new(min.x, min.y, min.z),
            max: Point3::new(max.x, max.y, max.z),
        }
    }

    pub fn union_with(&mut self, other: &AABB<N>) {
        // This is a tad verbose
        self.min = Point3::from_coordinates(cw_min3(&self.min.coords, &other.min.coords));
        self.max = Point3::from_coordinates(cw_max3(&self.max.coords, &other.max.coords));
    }

    pub fn empty() -> AABB<N> {
        AABB {
            max: Point3::new(N::min_value(), N::min_value(), N::min_value()),
            min: Point3::new(N::max_value(), N::max_value(), N::max_value()),
        }
    }
}
