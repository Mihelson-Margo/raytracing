use glm::Vec3;

use super::{Figure, PositionedFigure};

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
