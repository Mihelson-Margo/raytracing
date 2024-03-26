use glm::{vec3, Vec3};
use na::Matrix3;
use rand::{rngs::ThreadRng, Rng};
use std::f32::consts::PI;

use crate::objects::{LightSource, RayIntersection};
use crate::ray::Ray;

const EPS: f32 = 1e-4;

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

    pub fn pdf(n: &Vec3, d: &Vec3) -> f32 {
        if glm::dot(&d, n) <= 0.0 {
            0.0
        } else {
            0.5 / PI
        }
    }
}

impl Cosine {
    pub fn sample(n: &Vec3, rng: &mut ThreadRng) -> Vec3 {
        let theta = rng.gen_range(0.0..2.0 * PI);
        let r = rng.gen_range(0.0_f32..1.0).sqrt();

        let x = r * theta.cos();
        let y = r * theta.sin();
        let z = (1.0 - x * x - y * y).sqrt();

        let z_image = *n;
        let min_abs_coord = n.x.abs().min(n.y.abs()).min(n.z.abs());
        let x_image =
            Vec3::from_iterator(
                n.iter()
                    .map(|x| if x.abs() > min_abs_coord { 0.0 } else { 1.0 }),
            );
        let x_image = (x_image - n * glm::dot(&x_image, &z_image)).normalize();
        let y_image = glm::cross(&x_image, &z_image).normalize();

        let rot = Matrix3::from_columns(&[x_image, y_image, z_image]);
        rot * vec3(x, y, z)
    }

    pub fn pdf(n: &Vec3, d: &Vec3) -> f32 {
        glm::dot(n, d).max(0.0) / PI
    }
}

fn sphere_uniform(rng: &mut ThreadRng) -> Vec3 {
    let phi = rng.gen_range(0.0..PI);
    let z = rng.gen_range(-1.0_f32..1.0);
    let x = (1.0 - z * z).sqrt() * phi.cos();
    let y = (1.0 - z * z).sqrt() * phi.sin();
    vec3(x, y, z)
}

pub struct ToLight<'a> {
    pub lights: &'a [Box<dyn LightSource>],
}

impl<'a> ToLight<'a> {
    pub fn sample(&self, p: &Vec3, rng: &mut ThreadRng) -> Vec3 {
        assert!(!self.lights.is_empty());

        let idx = rng.gen_range(0..self.lights.len());
        let obj = &self.lights[idx];
        let p_light = obj.sample(rng);

        (p_light - p).normalize()
    }

    pub fn pdf(&self, p: &Vec3, d: &Vec3) -> f32 {
        if self.lights.is_empty() {
            return 0.0;
        }

        let ray = Ray::new(*p, *d);
        let mut pdf = 0.0;

        for obj in self.lights.iter() {
            let Some(i1) = obj.intersect(&ray) else {
                continue;
            };
            pdf += calc_intersection_pdf(obj, &ray, &i1, p);

            let ray2 = Ray::new_shifted(
                ray.origin + i1.t * ray.direction, ray.direction
            );

            let Some(i2) = obj.intersect(&ray2) else {
                continue;
            };
            pdf += calc_intersection_pdf(obj, &ray2, &i2, p);
        }

        pdf /= self.lights.len() as f32;
        pdf
    }
}

fn calc_intersection_pdf(
    obj: &Box<dyn LightSource>,
    ray: &Ray,
    intersection: &RayIntersection,
    initial_point: &Vec3,
) -> f32 {
    let obj_point = ray.origin + intersection.t * ray.direction;
    let dist = glm::length2(&(initial_point - obj_point));
    let cos = glm::dot(&ray.direction, &intersection.n).abs();

    let mut pdf = obj.pdf(&obj_point) * dist / cos;
    if !pdf.is_finite() {
        pdf = 0.0;
    }

    pdf
}

pub struct MIS<'a> {
    pub to_light: ToLight<'a>,
}

impl<'a> MIS<'a> {
    pub fn sample(&self, p: &Vec3, n: &Vec3, rng: &mut ThreadRng) -> Vec3 {
        if rng.gen_bool(self.cosine_probability()) {
            Cosine::sample(n, rng)
        } else {
            self.to_light.sample(p, rng)
        }
    }

    pub fn pdf(&self, p: &Vec3, n: &Vec3, d: &Vec3) -> f32 {
        let a = self.cosine_probability() as f32;
        let mut pdf =
            Cosine::pdf(n, &d) * a + self.to_light.pdf(p, &d) * (1.0 - a);

        // if !(pdf > 0.0) {
        //     pdf = f32::INFINITY;
        // }
        pdf
    }

    fn cosine_probability(&self) -> f64 {
        if self.to_light.lights.is_empty() {
            1.0
        } else {
            0.5
        }
    }
}
