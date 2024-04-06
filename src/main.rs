mod bvh;
mod camera;
mod image;
mod objects;
mod parser;
mod random;
mod ray;
mod trace;

use std::iter::repeat;
use std::cell::RefCell;

use image::Image;
use parser::*;
use rand::{rngs::ThreadRng, Rng};
use trace::trace_ray;

extern crate num_cpus;

fn render(scene: &Scene, image: &mut Image) {
    let n_cpus = num_cpus::get();
    println!("n_cpus = {}", n_cpus);

    let mut pool = simple_parallel::Pool::new(n_cpus);

    thread_local! {
        pub static RNG: RefCell<ThreadRng> = RefCell::new(rand::thread_rng());
    }

    for step in 0..scene.n_samples {
        pool.for_(image.data.iter_mut().enumerate(), |(ii, pixel)| {
            // for i in 0..scene.image.width {
            //     for j in 0..scene.image.height {
            let i = ii % image.width;
            let j = image.height - 1 - ii / image.width;

            //let mut rng = rand::thread_rng();
            RNG.with_borrow_mut(|mut rng| {
                let du = rng.gen::<f32>();
                let dv = rng.gen::<f32>();
                let u = (i as f32 + du) / image.width as f32 * 2.0 - 1.0;
                let v = (j as f32 + dv) / image.height as f32 * 2.0 - 1.0;
                let ray = scene.camera.ray_to_point(u, v);

                let old_color = *pixel;
                let color = trace_ray(scene, &ray, 0, &mut rng);
                let step_f = step as f32;
                let new_color = (old_color * step_f + color) / (step_f + 1.0);
                *pixel = new_color;
            });
            // scene.image.set(i, j, new_color);
            // }
            // }
        })
    }
}

fn main() {
    let input = std::env::args().nth(1).unwrap_or("assets/scene.txt".into());
    let output = std::env::args().nth(2).unwrap_or("/tmp/out.ppm".into());

    let (scene, mut image) = parse_scene(&input);
    render(&scene, &mut image);

    image.color_correction();
    image.write(&output);
}
