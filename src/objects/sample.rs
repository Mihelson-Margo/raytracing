use std::f32::consts::PI;

use glm::{vec3, Vec3};
use itertools::izip;
use rand::{rngs::ThreadRng, Rng};

use super::{Ellipsoid, Parallelipiped, PositionedFigure};

pub trait Sample {
    fn sample(&self, rng: &mut ThreadRng) -> Vec3;
    fn pdf(&self, p: &Vec3) -> f32;
}

impl<F: Sample> Sample for PositionedFigure<F> {
    fn sample(&self, rng: &mut ThreadRng) -> Vec3 {
        let point = self.figure.sample(rng);
        self.rotation * point + self.position
    }

    fn pdf(&self, p: &Vec3) -> f32 {
        let q = self.rotation.inverse() * (p - self.position);
        self.figure.pdf(&q)
    }
}

impl Sample for Parallelipiped {
    fn sample(&self, rng: &mut ThreadRng) -> Vec3 {
        let (a, b, c) = (2.0 * self.sizes.x, 2.0 * self.sizes.y, 2.0 * self.sizes.z);
        let area = 2.0 * (a * b + b * c + a * c);
        let sections = vec![
            a * b,
            2.0 * a * b,
            2.0 * a * b + a * c,
            2.0 * (a * b + a * c),
            area - b * c,
            area,
        ];
        let axis = vec![
            Vec3::z(),
            -Vec3::z(),
            Vec3::y(),
            -Vec3::y(),
            Vec3::x(),
            -Vec3::x(),
        ];

        let x = rng.gen_range(0.0..area);
        let mut p = izip!(&sections, axis)
            .filter(|(&c, _)| x <= c)
            .max_by(|(&c1, _), (c2, _)| c1.partial_cmp(c2).unwrap())
            .unwrap()
            .1;
        p = p.component_mul(&self.sizes);

        for i in 0..3 {
            if p[i] < 0.5 {
                p[i] = rng.gen_range(-self.sizes[i]..self.sizes[i]);
            }
        }

        p
    }

    fn pdf(&self, _p: &Vec3) -> f32 {
        let (a, b, c) = (self.sizes.x, self.sizes.y, self.sizes.z);
        let area = 4.0 * (a * b + b * c + a * c);
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

// TODO: remove copy paste
fn sphere_uniform(rng: &mut ThreadRng) -> Vec3 {
    let phi = rng.gen::<f32>() * std::f32::consts::PI;
    let z = rng.gen::<f32>() * 2.0 - 1.0;
    let x = (1.0 - z * z).sqrt() * phi.cos();
    let y = (1.0 - z * z).sqrt() * phi.sin();

    vec3(x, y, z)
}
