use glm::{vec3, Vec3};
use na::SimdPartialOrd;
use std::cell::RefCell;
use std::fs::File;
use std::io::Write;
use std::sync::{Arc, RwLock};

pub struct Image {
    pub width: usize,
    pub height: usize,
    pub data: Vec<Vec3>,
}

impl Image {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            data: (0..width * height)
                .map(|_| Vec3::zeros())
                .collect::<Vec<_>>(),
        }
    }

    // pub fn get(&self, u: usize, v: usize) -> Vec3 {
    //     let v = self.height - 1 - v;
    //     self.data[self.width * v + u]
    // }

    // pub fn set(&mut self, u: usize, v: usize, color: Vec3) {
    //     let v = self.height - 1 - v;
    //     self.data[self.width * v + u] = color;
    // }

    pub fn write(&self, path: &str) {
        let mut file = File::create(path).unwrap();
        file.write_all("P6\n".as_bytes()).unwrap();
        file.write_all(format!("{} {}\n", self.width, self.height).as_bytes())
            .unwrap();
        file.write_all("255\n".as_bytes()).unwrap();

        let data = self
            .data
            .iter()
            .flat_map(|color| {
                [color.x, color.y, color.z]
                    .into_iter()
                    .map(|x| (255.0 * x).round() as u8)
            })
            .collect::<Vec<_>>();

        file.write_all(&data).unwrap();
    }

    pub fn color_correction(&mut self) {
        for color in &mut self.data {
            let c = aces_tonemap(color);
            let c = gamma_correction(&c);
            *color = c;
        }
    }
}

fn gamma_correction(color: &Vec3) -> Vec3 {
    let pow = 1.0 / 2.2;
    Vec3::from_iterator(color.iter().map(|x| x.powf(pow)))
}

fn aces_tonemap(x: &Vec3) -> Vec3 {
    const A: f32 = 2.51;
    const B: f32 = 0.03;
    const C: f32 = 2.43;
    const D: f32 = 0.59;
    const E: f32 = 0.14;

    let up = (A * x).add_scalar(B);
    let up = x.component_mul(&up);

    let down = (C * x).add_scalar(D);
    let down = x.component_mul(&down).add_scalar(E);

    saturate(up.component_div(&down))
}

fn saturate(color: Vec3) -> Vec3 {
    color.simd_clamp(Vec3::zeros(), vec3(1.0, 1.0, 1.0))
}
