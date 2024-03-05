use glm::{vec3, Vec3};
use rand::{rngs::ThreadRng, Rng};

pub fn rand_direction(rng: &mut ThreadRng, normal: &Vec3) -> Vec3 {
    let phi = rng.gen::<f32>() * std::f32::consts::PI;
    let z = rng.gen::<f32>() * 2.0 - 1.0;
    let x = (1.0 - z * z).sqrt() * phi.cos();
    let y = (1.0 - z * z).sqrt() * phi.sin();

    let mut d = vec3(x, y, z);
    if glm::dot(&d, normal) < 0.0 {
        d = -d;
    }
    d
}
