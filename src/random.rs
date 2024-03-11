use glm::{vec3, Vec3};
use rand::{rngs::ThreadRng, Rng};
use std::f32::consts::PI;

use crate::{
    objects::{Geometry, LightSource, PositionedFigure, Sample},
    ray::Ray,
};


const EPS: f32 = 1e-4;

pub struct SampledDirection {
    pub d: Vec3,
    pub pdf: f32,
}

pub struct Uniform;
pub struct Cosine;

impl Uniform {
    pub fn sample(n: &Vec3, rng: &mut ThreadRng) -> Vec3 {
        let mut d = sphere_uniform(rng);
        if glm::dot(&d, n) <= 0.0 {
            d = -d;
        };
        d
    }

    pub fn pdf(_n: &Vec3, _d: &Vec3) -> f32 {
        0.5 / PI
    }
}

impl Cosine {
    pub fn sample(n: &Vec3, rng: &mut ThreadRng) -> Vec3 {
        // TODO: handle d = -n
        let d = sphere_uniform(rng).normalize();
        (d + n.normalize()*(1.0 + EPS)).normalize()
    }

    pub fn pdf(n: &Vec3, d: &Vec3) -> f32 {
        glm::dot(&n.normalize(), &d.normalize()).max(0.0) / PI
    }
}

fn sphere_uniform(rng: &mut ThreadRng) -> Vec3 {
    // let phi = rng.gen::<f32>() * PI;
    // let z = rng.gen::<f32>() * 2.0 - 1.0;
    let phi = rng.gen_range(0.0..PI);
    let z = rng.gen_range(-1.0_f32..1.0);
    let x = (1.0 - z * z).sqrt() * phi.cos();
    let y = (1.0 - z * z).sqrt() * phi.sin();

    vec3(x, y, z)
}

pub struct ToLigth<'a> {
    pub lights: &'a [Box<dyn LightSource>],
}

impl<'a> ToLigth<'a> {
    pub fn sample(&self, p: &Vec3, n: &Vec3, rng: &mut ThreadRng) -> Vec3 {
        if self.lights.is_empty() {
            return Uniform::sample(n, rng);
        }

        let idx = rng.gen_range(0..self.lights.len());
        let obj = &self.lights[idx];
        let p_light = obj.sample(rng);

        (p_light - p).normalize()
    }

    pub fn pdf(&self, p: &Vec3, n: &Vec3, d: &Vec3) -> f32 {
        if self.lights.is_empty() {
            return Uniform::pdf(n, d);
        }

        let ray = Ray::new(*p, *d);

        // let t0 = glm::length(&(p_light - p));
        // println!("=========");
        // println!("o + t*d = {} + {}*{} = {} =? {}", ray.origin, t0, ray.direction,
        //     ray.origin + t0*ray.direction, p_light);

        // println!("Origin: {}, light: {}", p, p_light);

        let mut pdf = 0.0;

        for obj in self.lights.iter() {
            let Some(i1) = obj.intersect(&ray) else {
                continue;
            };
            let q1 = ray.origin + i1.t * ray.direction;
            let ray2 = Ray::new_shifted(q1, ray.direction);

            let i2 = obj.intersect(&ray2).unwrap_or(i1.clone());
            let q2 = ray2.origin + i2.t * ray2.direction;

            let pdf1 =
                obj.pdf(&q1) * glm::length2(&(p - q1)) / glm::dot(&ray.direction, &i1.n).abs();
            let pdf2 =
                obj.pdf(&q2) * glm::length2(&(p - q2)) / glm::dot(&ray.direction, &i2.n).abs();

            // println!("n1 = {}, n2 = {}", i1.n, i2.n);
            // println!("pdf1 = {}, pdf2 = {}", pdf1, pdf2);
            // assert!(obj.pdf(&q1) > 0.0 && obj.pdf(&q2) > 0.0);

            // let l1 = glm::length(&(p_light - q1));
            // let l2 = glm::length(&(p_light - q2));
            // println!("q1 = {}, q2 = {}, l1 = {}, l2 = {}", q1, q2, l1, l2);
            // assert!(l1 < 0.1 || l2 < 0.1);

            if pdf1.is_finite() {
                pdf += pdf1;
            }
            if pdf2.is_finite() {
                pdf += pdf2;
            }

            assert!(pdf >= 0.0);
        }

        pdf /= self.lights.len() as f32;
        pdf
    }
}



pub struct MIS<'a> {
    pub to_light: ToLigth<'a>,
}

impl<'a> MIS<'a> {
    pub fn sample(&self, p: &Vec3, n: &Vec3, rng: &mut ThreadRng) -> SampledDirection {
        let cosine_prob = 0.5_f32;
        let d = if rng.gen_bool(cosine_prob as f64) {
            Uniform::sample(n, rng)
        } else {
            self.to_light.sample(p, n, rng)
        };

        let mut pdf =
            Uniform::pdf(n, &d) * cosine_prob + self.to_light.pdf(p, n, &d) * (1.0 - cosine_prob);

        if !(pdf > 0.0) {
            pdf = f32::INFINITY;
        }
        // println!("pdf = {}", pdf);
        assert!(pdf > 0.0);

        SampledDirection { d, pdf }
    }
}
