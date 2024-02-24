mod camera;
mod image;
mod light;
mod objects;
mod parser;
mod ray;
mod trace;

use parser::*;
use trace::trace_ray;

fn render(scene: &mut Scene) {
    for i in 0..scene.image.width {
        for j in 0..scene.image.height {
            let u = (i as f32 + 0.5) / scene.image.width as f32 * 2.0 - 1.0;
            let v = (j as f32 + 0.5) / scene.image.height as f32 * 2.0 - 1.0;
            let ray = scene.camera.ray_to_point(u, v);

            if let Some(color) = trace_ray(scene, &ray, 0) {
                scene.image.set(i, j, color);
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
