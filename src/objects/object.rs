use glm::Vec3;

use super::{Aabb, Figure, Geometry, PositionedFigure, RayIntersection};

pub enum Material {
    Diffuse,
    Metallic,
    Dielectric { ior: f32 },
}

pub struct Object {
    pub geometry: PositionedFigure,

    pub color: Vec3,
    pub emission: Vec3,
    pub material: Material,
}

impl Object {
    pub fn new(geometry: Figure) -> Self {
        Self {
            geometry: PositionedFigure::new(geometry),
            color: Vec3::zeros(),
            emission: Vec3::zeros(),
            material: Material::Diffuse,
        }
    }
}

impl Geometry for Object {
    fn intersect(&self, ray: &crate::ray::Ray) -> Option<RayIntersection> {
        self.geometry.intersect(ray)
    }

    fn calc_aabb(&self) -> Aabb {
        self.geometry.calc_aabb()
    }
}
