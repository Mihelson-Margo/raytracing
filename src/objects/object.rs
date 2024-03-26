use glm::Vec3;

use super::PositionedFigure;

pub enum Material {
    Diffuse,
    Metallic,
    Dielectric { ior: f32 },
}

pub struct Object<G> {
    pub geometry: PositionedFigure<G>,

    pub color: Vec3,
    pub emission: Vec3,
    pub material: Material,
}

impl<G> Object<G> {
    pub fn new(geometry: G) -> Self {
        Self {
            geometry: PositionedFigure::new(geometry),
            color: Vec3::zeros(),
            emission: Vec3::zeros(),
            material: Material::Diffuse,
        }
    }
}
