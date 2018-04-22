use aabb::{partial_max, AABB};
use nalgebra::*;
use std::f32;

#[derive(Copy, Clone, Debug, Default)]
pub struct Frustum {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
    // near clip plane position
    pub near_plane: f32,
    // far clip plane position
    pub far_plane: f32,
}

/// Represents a camera (a view of a scene).
#[derive(Copy, Clone, Debug)]
pub struct Camera {
    // Projection parameters
    // frustum (for culling)
    pub frustum: Frustum,
    // view matrix
    // (World -> View)
    pub view: Isometry3<f32>,
    // projection matrix
    // (View -> clip?)
    pub projection: Perspective3<f32>,
}

/// A camera controller that generates `Camera` instances.
///
/// TODO describe parameters
#[derive(Clone, Debug)]
pub struct CameraControl {
    fovy: f32,
    aspect_ratio: f32,
    near_plane: f32,
    far_plane: f32,
    zoom: f32,
    orbit_radius: f32,
    theta: f32,
    phi: f32,
    target: Point3<f32>,
}

impl Default for CameraControl {
    fn default() -> CameraControl {
        CameraControl {
            fovy: f32::consts::PI / 2.0,
            aspect_ratio: 1.0,
            near_plane: 0.001,
            far_plane: 10.0,
            zoom: 1.0,
            orbit_radius: 1.0,
            theta: 0.0,
            phi: f32::consts::PI / 2.0,
            target: Point3::new(0.0, 0.0, 0.0),
        }
    }
}

impl CameraControl {
    pub fn set_aspect_ratio(&mut self, aspect_ratio: f32) {
        self.aspect_ratio = aspect_ratio;
    }

    /// Centers the camera on the given axis-aligned bounding box.
    /// Orbit angles are not reset.
    pub fn center_on_aabb(&mut self, aabb: AABB<f32>, fovy: f32) {
        let size = {
            let size = aabb.size();
            partial_max(size.x, partial_max(size.y, size.z))
        };

        let cx = (aabb.max.x + aabb.min.x) / 2.0;
        let cy = (aabb.max.y + aabb.min.y) / 2.0;
        let cz = (aabb.max.z + aabb.min.z) / 2.0;
        let center = Point3::new(cx, cy, cz);
        let cam_dist = (0.5 * size) / f32::tan(0.5 * fovy);

        self.orbit_radius = cam_dist;
        self.target = center;
        self.near_plane = 0.1 * cam_dist;
        self.far_plane = 10.0 * cam_dist;
        self.fovy = fovy;

        //debug!("Center on AABB: {:?} -> {:?}", &aabb, &self);
    }

    fn orbit_to_cartesian(&self) -> Vector3<f32> {
        Vector3::new(
            self.orbit_radius * f32::sin(self.theta) * f32::sin(self.phi),
            self.orbit_radius * f32::cos(self.phi),
            self.orbit_radius * f32::cos(self.theta) * f32::sin(self.phi),
        )
    }

    fn get_look_at(&self) -> Isometry3<f32> {
        let dir = self.orbit_to_cartesian();
        Isometry3::look_at_rh(&(self.target + dir), &self.target, &Vector3::y())
    }

    /// Returns a `Camera` for the current viewpoint.
    pub fn camera(&self) -> Camera {
        Camera {
            frustum: Default::default(),
            view: self.get_look_at(),
            projection: Perspective3::new(
                self.aspect_ratio,
                self.fovy,
                self.near_plane,
                self.far_plane,
            ),
        }
    }
}
