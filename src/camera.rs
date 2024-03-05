use glm::{vec3, Vec3};
use na::Matrix3;

use crate::ray::Ray;

pub struct Camera {
    pub position: Vec3,
    pub axis: Matrix3<f32>,

    pub tg_fov_x: f32,
    pub tg_fov_y: f32,
}

impl Camera {
    pub fn ray_to_point(&self, u: f32, v: f32) -> Ray {
        assert!(u.abs() <= 1.0 && v.abs() <= 1.0);

        let direction = vec3(u * self.tg_fov_x, v * self.tg_fov_y, 1.0);
        let direction = self.axis * direction;

        Ray::new(self.position, direction)
    }
}
