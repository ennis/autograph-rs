use nalgebra::{Scalar, Vector3, Vector4, Point3, Affine3, min, max, Matrix, U1, U2, U3, U4};
use nalgebra::storage::{Storage, OwnedStorage};
use alga::general::{Field};
use std::cmp::Ord;

pub struct AABB<N: Scalar + Field>
{
    min: Point3<N>,
    max: Point3<N>
}

fn cw_min3<N: Scalar + Ord>(a: &Vector3<N>, b: &Vector3<N>) -> Vector3<N>
{
    Vector3::new(min(a.x, b.x), min(a.y, b.y), min(a.z, b.z))
}

fn cw_max3<N: Scalar + Ord>(a: &Vector3<N>, b: &Vector3<N>) -> Vector3<N>
{
    Vector3::new(max(a.x, b.x), max(a.y, b.y), max(a.z, b.z))
}

fn cw_min4<N: Scalar + Ord>(a: &Vector4<N>, b: &Vector4<N>) -> Vector4<N>
{
    Vector4::new(min(a.x, b.x), min(a.y, b.y), min(a.z, b.z), min(a.w, b.w))
}

fn cw_max4<N: Scalar + Ord>(a: &Vector4<N>, b: &Vector4<N>) -> Vector4<N>
{
    Vector4::new(max(a.x, b.x), max(a.y, b.y), max(a.z, b.z), max(a.w, b.w))
}

impl<N: Scalar + Field + Ord> AABB<N>
{
    pub fn size(&self) -> Vector3<N> {
        self.max - self.min
    }

    /// Reference:
    /// http://dev.theomader.com/transform-bounding-boxes/
    pub fn transform(&self, tr: &Affine3<N>) -> AABB<N> {
        let trh = tr.to_homogeneous();
        let xa = trh.column(0) * self.min[0];
        let xb = trh.column(0) * self.max[0];
        let ya = trh.column(1) * self.min[1];
        let yb = trh.column(1) * self.max[1];
        let za = trh.column(2) * self.min[2];
        let zb = trh.column(2) * self.max[2];

        let min = cw_min4(&xa, &xb) + cw_min4(&ya, &yb) + cw_min4(&za, &zb) + trh.column(3);
        let max = cw_max4(&xa, &xb) + cw_max4(&ya, &yb) + cw_max4(&za, &zb) + trh.column(3);

        AABB { min: Point3::new(min.x,min.y,min.z), max: Point3::new(max.x,max.y,max.z) }
    }

    pub fn union_with(&mut self, other: &AABB<N>)
    {
        // This is a tad verbose
        self.min = Point3::from_coordinates(cw_min3(&self.min.coords, &other.min.coords));
        self.max = Point3::from_coordinates(cw_max3(&self.max.coords, &other.max.coords));
    }
}