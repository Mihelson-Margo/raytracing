use glm::Vec3;

const EPS: f32 = 1e-4;

pub struct Ray {
    pub origin: Vec3,
    pub direction: Vec3,
}

impl Ray {
    pub fn new(origin: Vec3, direction: Vec3) -> Self {
        Self {
            origin,
            direction: direction.normalize(),
        }
    }

    pub fn new_shifted(origin: Vec3, direction: Vec3) -> Self {
        let direction = direction.normalize();
        Self {
            origin: origin + EPS * direction,
            direction,
        }
    }
}
