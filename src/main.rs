mod bvh;
mod camera;
mod image;
mod objects;
mod parser;
mod random;
mod ray;
mod trace;

use std::{iter::repeat, sync::Arc};

use glm::{vec3, Vec3};
use rand::{rngs::ThreadRng, Rng};
use rayon::prelude::*;

use image::Image;
use parser::*;
use trace::trace_ray;

extern crate num_cpus;

fn render(scene: Arc<Scene>, parameters: &Parameters) -> Image {
    let n_cpus = num_cpus::get();
    println!("n_cpus = {}", n_cpus);

    let w = parameters.image_width;
    let h = parameters.image_height;

    let pixels = (0..w * h)
        .collect::<Vec<_>>()
        .par_iter()
        .map(|ii| {
            let i = ii % w;
            let j = h - 1 - ii / w;

            let mut rng = rand::thread_rng();
            let mut total_color = glm::Vec3::zeros();

            for step in 0..parameters.n_samples {
                let du = rng.gen::<f32>();
                let dv = rng.gen::<f32>();
                let u = (i as f32 + du) / w as f32 * 2.0 - 1.0;
                let v = (j as f32 + dv) / h as f32 * 2.0 - 1.0;
                let ray = scene.camera.ray_to_point(u, v);

                let old_color = total_color;
                let color = trace_ray(&scene, parameters, &ray, 0, &mut rng);
                let step_f = step as f32;
                total_color = (old_color * step_f + color) / (step_f + 1.0);
            }
            total_color
        })
        .collect::<Vec<_>>();

    // TODO: fix
    let mut image = Image::new(w, h);
    image.data = pixels;
    image
}

fn main() {
    let input = std::env::args()
        .nth(1)
        .unwrap_or("assets/practice6_1.gltf".into());
    let width = parse_arg_usize(2, 500);
    let height = parse_arg_usize(3, 500);
    let samples = parse_arg_usize(4, 16);
    let output = std::env::args().nth(5).unwrap_or("/tmp/out.ppm".into());

    let scene = parse_scene(&input, width, height);
    let scene = Arc::new(scene);

    let parameters = Parameters {
        ray_depth: 6,
        n_samples: samples,
        image_width: width,
        image_height: height,
        background_color: Vec3::zeros(), // + vec3(0.03, 0.03, 0.03),
    };

    let mut image = render(scene, &parameters);

    image.color_correction();
    image.write(&output);
}

fn parse_arg_usize(arg_idx: usize, default: usize) -> usize {
    std::env::args()
        .nth(arg_idx)
        .map(|x| x.parse::<usize>().unwrap())
        .unwrap_or(default)
}
