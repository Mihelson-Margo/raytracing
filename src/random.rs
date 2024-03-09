use glm::{vec3, Vec3};
use rand::{rngs::ThreadRng, Rng};

use crate::{
    objects::{Geometry, LightSource, PositionedFigure, Sample},
    ray::Ray,
};

pub struct SampledDirection {
    pub d: Vec3,
    pub pdf: f32,
}

pub struct Uniform;
pub struct Cosine;

impl Uniform {
    pub fn sample(n: &Vec3, rng: &mut ThreadRng) -> Vec3 {
        let d = sphere_uniform(rng);

        if glm::dot(&d, n) >= 0.0 {
            d
        } else {
            -d
        }
    }

    pub fn pdf() -> f32 {
        0.5 / std::f32::consts::PI
    }
}

impl Cosine {
    pub fn sample(n: &Vec3, rng: &mut ThreadRng) -> Vec3 {
        // TODO: handle d = -n
        let d = sphere_uniform(rng);
        (d + n).normalize()
    }

    pub fn pdf(n: &Vec3, d: &Vec3) -> f32 {
        glm::dot(n, d) / std::f32::consts::PI
    }
}

fn sphere_uniform(rng: &mut ThreadRng) -> Vec3 {
    let phi = rng.gen::<f32>() * std::f32::consts::PI;
    let z = rng.gen::<f32>() * 2.0 - 1.0;
    let x = (1.0 - z * z).sqrt() * phi.cos();
    let y = (1.0 - z * z).sqrt() * phi.sin();

    vec3(x, y, z)
}

pub struct ToLigth<'a> {
    pub lights: &'a [Box<dyn LightSource>],
}

impl<'a> ToLigth<'a> {
    pub fn sample(&self, p: &Vec3, rng: &mut ThreadRng) -> SampledDirection {
        let n = self.lights.len();
        let idx = rng.gen_range(0..n);
        let p_light = self.lights[idx].sample(rng);
        let ray = Ray::new_shifted(*p, p_light - p);

        let Some(i1) = self.lights[idx].intersect(&ray) else {
            return SampledDirection {
                d: ray.direction,
                pdf: 0.0,
            };
        };
        let q1 = ray.origin + i1.t * ray.direction;
        let i2 = self.lights[idx]
            .intersect(&Ray::new_shifted(q1, ray.direction))
            .unwrap_or(i1.clone());
        let q2 = q1 + i2.t * ray.direction;
        let pdf1 = self.lights[idx].pdf(&q1) * glm::length2(&(p - q1))
            / glm::dot(&ray.direction, &i1.n).abs();
        let pdf2 = self.lights[idx].pdf(&q2) * glm::length2(&(p - q2))
            / glm::dot(&ray.direction, &i2.n).abs();

        let pdf = pdf1 + pdf2;

        SampledDirection {
            d: ray.direction,
            pdf: pdf / n as f32,
        }
    }
}

pub struct MIS<'a> {
    pub to_light: ToLigth<'a>,
}

impl<'a> MIS<'a> {
    pub fn sample(&self, p: &Vec3, n: &Vec3, rng: &mut ThreadRng) -> SampledDirection {
        if rng.gen_bool(0.5) {
            let d = Cosine::sample(n, rng);
            let pdf = Cosine::pdf(n, &d);
            SampledDirection { d, pdf: pdf / 2.0 }
        } else {
            let res = self.to_light.sample(p, rng);
            SampledDirection {
                d: res.d,
                pdf: res.pdf / 2.0,
            }
        }
    }
}
