use glm::{vec3, Vec3};

pub trait Light {
    fn intensity(&self, p: &Vec3) -> Vec3;
    fn direction_to_light(&self, p: &Vec3) -> Vec3;
    fn dist_to_light(&self, p: &Vec3) -> f32;
}

pub struct DirectedLight {
    pub direction: Vec3,
    pub intensity: Vec3,
}

impl Light for DirectedLight {
    fn direction_to_light(&self, _p: &Vec3) -> Vec3 {
        self.direction
    }

    fn intensity(&self, _p: &Vec3) -> Vec3 {
        self.intensity
    }

    fn dist_to_light(&self, _p: &Vec3) -> f32 {
        f32::INFINITY
    }
}

pub struct PointLight {
    pub position: Vec3,
    pub intensity: Vec3,
    pub attenuation: Vec3,
}

impl Light for PointLight {
    fn direction_to_light(&self, p: &Vec3) -> Vec3 {
        self.position - p
    }

    fn intensity(&self, p: &Vec3) -> Vec3 {
        let r = glm::length(&(self.position - p));
        let denom = glm::dot(&self.attenuation, &vec3(1.0, r, r * r));
        self.intensity / denom
    }

    fn dist_to_light(&self, p: &Vec3) -> f32 {
        glm::length(&(self.position - p))
    }
}
