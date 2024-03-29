use std::f32::consts::PI;

use glm::{vec3, Vec3};
use rand::{rngs::ThreadRng, Rng};

use super::{Ellipsoid, Figure, Parallelipiped, PositionedFigure, Triangle};

pub trait Sample {
    fn sample(&self, rng: &mut ThreadRng) -> Vec3;
    fn pdf(&self, p: &Vec3) -> f32;
}

impl Sample for PositionedFigure {
    fn sample(&self, rng: &mut ThreadRng) -> Vec3 {
        let point = self.figure.sample(rng);
        self.rotation * point + self.position
    }

    fn pdf(&self, p: &Vec3) -> f32 {
        let q = self.rotation.inverse() * (p - self.position);
        self.figure.pdf(&q)
    }
}

impl Sample for Figure {
    fn sample(&self, rng: &mut ThreadRng) -> Vec3 {
        match &self {
            Figure::Plane(_) => panic!(),
            Figure::Ellipsoid(ellipsoid) => ellipsoid.sample(rng),
            Figure::Parallelipiped(parallelipiped) => parallelipiped.sample(rng),
            Figure::Triangle(triangle) => triangle.sample(rng),
        }
    }

    fn pdf(&self, p: &Vec3) -> f32 {
        match &self {
            Figure::Plane(_) => panic!(),
            Figure::Ellipsoid(ellipsoid) => ellipsoid.pdf(p),
            Figure::Parallelipiped(parallelipiped) => parallelipiped.pdf(p),
            Figure::Triangle(triangle) => triangle.pdf(p),
        }
    }
}

impl Sample for Parallelipiped {
    fn sample(&self, rng: &mut ThreadRng) -> Vec3 {
        let (a, b, c) = (self.sizes.x, self.sizes.y, self.sizes.z);
        let area = a * b + b * c + a * c;

        let x = rng.gen_range(0.0..area);
        let mut p = if x < a * b {
            Vec3::z()
        } else if x < a * b + a * c {
            Vec3::y()
        } else {
            Vec3::z()
        };

        if rng.gen_bool(0.5) {
            p = -p;
        }
        p = p.component_mul(&self.sizes);

        for i in 0..3 {
            if p[i] == 0.0 {
                p[i] = rng.gen_range(-self.sizes[i]..self.sizes[i]);
            }
        }

        p
    }

    fn pdf(&self, _p: &Vec3) -> f32 {
        let (a, b, c) = (self.sizes.x, self.sizes.y, self.sizes.z);
        let area = 8.0 * (a * b + b * c + a * c);
        1.0 / area
    }
}

impl Sample for Ellipsoid {
    fn sample(&self, rng: &mut ThreadRng) -> Vec3 {
        let p_sphere = sphere_uniform(rng);
        p_sphere.component_mul(&self.radiuses)
    }

    fn pdf(&self, p: &Vec3) -> f32 {
        let n = p.component_div(&self.radiuses);
        let n = n.component_mul(&n);
        let r = self.radiuses.component_mul(&self.radiuses);

        let denom = n.x * r.y * r.z + r.x * n.y * r.z + r.x * r.y * n.z;

        1.0 / (4.0 * PI * denom.sqrt())
    }
}

impl Sample for Triangle {
    fn sample(&self, rng: &mut ThreadRng) -> Vec3 {
        let mut a = rng.gen_range(0.0..1.0);
        let mut b = rng.gen_range(0.0..1.0);
        if a + b > 1.0 {
            a = 1.0 - a;
            b = 1.0 - b;
        }

        self.v + a * self.edge1 + b * self.edge2
    }

    fn pdf(&self, _p: &Vec3) -> f32 {
        self.inv_area
    }
}

// TODO: remove copy paste
fn sphere_uniform(rng: &mut ThreadRng) -> Vec3 {
    let phi = rng.gen::<f32>() * std::f32::consts::PI;
    let z = rng.gen::<f32>() * 2.0 - 1.0;
    let x = (1.0 - z * z).sqrt() * phi.cos();
    let y = (1.0 - z * z).sqrt() * phi.sin();

    vec3(x, y, z)
}
