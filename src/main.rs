mod camera;
mod image;
mod objects;
mod parser;
mod random;
mod ray;
mod trace;

use parser::*;
use rand::Rng;
use trace::trace_ray;

fn render(scene: &mut Scene) {
    for step in 0..scene.n_samples {
        for i in 0..scene.image.width {
            for j in 0..scene.image.height {
                let du = scene.generator.gen::<f32>();
                let dv = scene.generator.gen::<f32>();
                let u = (i as f32 + du) / scene.image.width as f32 * 2.0 - 1.0;
                let v = (j as f32 + dv) / scene.image.height as f32 * 2.0 - 1.0;
                let ray = scene.camera.ray_to_point(u, v);

                let old_color = scene.image.get(i, j);
                let color = trace_ray(scene, &ray, 0);
                let step_f = step as f32;
                let new_color = (old_color * step_f + color) / (step_f + 1.0);
                scene.image.set(i, j, new_color);
            }
        }
    }
}

fn main() {
    let input = std::env::args().nth(1).unwrap_or("assets/scene.txt".into());
    let output = std::env::args().nth(2).unwrap_or("/tmp/out.ppm".into());

    let mut scene = parse_scene(&input);
    render(&mut scene);

    scene.image.color_correction();
    scene.image.write(&output);
}
