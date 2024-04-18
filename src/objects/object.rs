use glm::Vec3;

use super::{Aabb, Geometry, RayIntersection, Triangle};

pub enum MaterialType {
    Diffuse,
    Metallic,
    Dielectric { ior: f32 },
}

pub struct Material {
    pub color: Vec3,
    pub emission: Vec3,
    pub material_type: MaterialType,
}

pub struct Primitive {
    pub triangle: Triangle,
    pub material_idx: usize,
}

impl Geometry for Primitive {
    fn intersect(&self, ray: &crate::ray::Ray) -> Option<RayIntersection> {
        self.triangle.intersect(ray)
    }

    fn calc_aabb(&self) -> Aabb {
        self.triangle.calc_aabb()
    }
}
