use glm::Vec3;
use na::UnitQuaternion;

#[derive(Debug, Clone)]
pub struct Plane {
    // contains 0
    pub normal: Vec3,
}

#[derive(Debug, Clone)]
pub struct Ellipsoid {
    // center is 0
    pub radiuses: Vec3,
}

#[derive(Debug, Clone)]
pub struct Parallelipiped {
    // center is 0
    pub sizes: Vec3,
}

#[derive(Debug, Clone)]
pub struct Triangle {
    pub v: Vec3,
    pub edge1: Vec3,
    pub edge2: Vec3,
    pub normal: Vec3,
    pub inv_area: f32,
}

impl Triangle {
    pub fn new(v1: Vec3, v2: Vec3, v3: Vec3) -> Self {
        let edge1 = v2 - v1;
        let edge2 = v3 - v1;
        let normal = glm::cross(&edge1, &edge2);
        let area = glm::length(&normal) / 2.0;
        let normal = normal.normalize();

        Self {
            v: v1,
            edge1,
            edge2,
            normal,
            inv_area: 1.0 / area,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Figure {
    Plane(Plane),
    Ellipsoid(Ellipsoid),
    Parallelipiped(Parallelipiped),
    Triangle(Triangle),
}

#[derive(Debug, Clone)]
pub struct PositionedFigure {
    pub figure: Figure,
    pub position: Vec3,
    pub rotation: UnitQuaternion<f32>,
}

impl PositionedFigure {
    pub fn new(figure: Figure) -> Self {
        Self {
            figure,
            position: Vec3::zeros(),
            rotation: UnitQuaternion::identity(),
        }
    }
}
