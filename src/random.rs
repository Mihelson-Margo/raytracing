use glm::{vec3, Vec3};
use rand::{rngs::ThreadRng, Rng};
use std::f32::consts::PI;

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
    pub fn sample(n: &Vec3, rng: &mut ThreadRng) -> SampledDirection {
        let mut d = sphere_uniform(rng);
        if glm::dot(&d, n) <= 0.0 {
            d = -d;
        }

        SampledDirection { d, pdf: 0.5 / PI }
    }
}

impl Cosine {
    pub fn sample(n: &Vec3, rng: &mut ThreadRng) -> Vec3 {
        // TODO: handle d = -n
        let d = sphere_uniform(rng);
        (d + n).normalize()
    }

    pub fn pdf(n: &Vec3, d: &Vec3) -> f32 {
        glm::dot(n, d) / PI
    }
}

fn sphere_uniform(rng: &mut ThreadRng) -> Vec3 {
    let phi = rng.gen::<f32>() * PI;
    let z = rng.gen::<f32>() * 2.0 - 1.0;
    let x = (1.0 - z * z).sqrt() * phi.cos();
    let y = (1.0 - z * z).sqrt() * phi.sin();

    vec3(x, y, z)
}

pub struct ToLigth<'a> {
    pub lights: &'a [Box<dyn LightSource>],
}

impl<'a> ToLigth<'a> {
    pub fn sample(&self, p: &Vec3, n: &Vec3, rng: &mut ThreadRng) -> SampledDirection {
        if self.lights.is_empty() {
            return Uniform::sample(n, rng);
        }

        let idx = rng.gen_range(0..self.lights.len());
        let obj = &self.lights[idx];
        let p_light = obj.sample(rng);
        let ray = Ray::new(*p, p_light - p);

        let t0 = glm::length(&(p_light - p));
        // println!("=========");
        // println!("o + t*d = {} + {}*{} = {} =? {}", ray.origin, t0, ray.direction,
        //     ray.origin + t0*ray.direction, p_light);

        // println!("Origin: {}, light: {}", p, p_light);

        let Some(i1) = obj.intersect(&ray) else {
            // println!("Oooops");
            return SampledDirection {
                d: ray.direction,
                pdf: f32::INFINITY,
            };
        };
        let q1 = ray.origin + i1.t * ray.direction;
        let ray2 = Ray::new_shifted(q1, ray.direction);

        let i2 = obj
            .intersect(&ray2)
            .unwrap_or(i1.clone());
        let q2 = ray2.origin + i2.t * ray2.direction;

        let pdf1 = obj.pdf(&q1) * glm::length2(&(p - q1))
            / glm::dot(&ray.direction, &i1.n).abs();
        let pdf2 = obj.pdf(&q2) * glm::length2(&(p - q2))
            / glm::dot(&ray.direction, &i2.n).abs();

        // println!("n1 = {}, n2 = {}", i1.n, i2.n);
        // println!("pdf1 = {}, pdf2 = {}", pdf1, pdf2);
        // assert!(obj.pdf(&q1) > 0.0 && obj.pdf(&q2) > 0.0);

        let l1 = glm::length(&(p_light - q1));
        let l2 = glm::length(&(p_light - q2));
        // println!("q1 = {}, q2 = {}, l1 = {}, l2 = {}", q1, q2, l1, l2);
        // assert!(l1 < 0.1 || l2 < 0.1);

        let mut pdf = pdf1 + pdf2;
        if (l1 > 0.01 && l2 > 0.01) {
            pdf = f32::INFINITY;
        }

        SampledDirection {
            d: ray.direction,
            pdf: pdf / self.lights.len() as f32,
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
            let res = self.to_light.sample(p, n, rng);
            SampledDirection {
                d: res.d,
                pdf: res.pdf / 2.0,
            }
        }
    }
}
