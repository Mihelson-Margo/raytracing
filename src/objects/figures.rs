use glm::Vec3;
use na::UnitQuaternion;

pub struct Plane {
    // contains 0
    pub normal: Vec3,
}

pub struct Ellipsoid {
    // center is 0
    pub radiuses: Vec3,
}

pub struct Parallelipiped {
    // center is 0
    pub sizes: Vec3,
}

pub struct PositionedFigure<F> {
    pub figure: F,
    pub position: Vec3,
    pub rotation: UnitQuaternion<f32>,
}

impl<F> PositionedFigure<F> {
    pub fn new(figure: F) -> Self {
        Self {
            figure,
            position: Vec3::zeros(),
            rotation: UnitQuaternion::identity(),
        }
    }
}
