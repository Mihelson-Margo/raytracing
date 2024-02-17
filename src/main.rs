mod camera;
mod image;
mod objects;
mod parser;

use parser::*;


fn render(scene: &mut Scene) {
    for i in 0..scene.image.width {
        for j in 0..scene.image.height {
            let u = (i as f32 + 0.5) / scene.image.width as f32 * 2.0 - 1.0;
            let v = (j as f32 + 0.5) / scene.image.height as f32 * 2.0 - 1.0;

            let ray = scene.camera.ray_to_point(u, v);

            let intersection = scene.objects.iter().filter_map(|object|{
                object.intersect(&ray)
            })
            .min_by(|a, b| a.t.partial_cmp(&b.t).unwrap());

            if let Some(intersection) = intersection {
                scene.image.set(i, j, intersection.color);
            }
        }
    }
}


fn main() {
    let input = std::env::args().nth(1).unwrap_or("assets/scene.txt".into());
    let output = std::env::args().nth(2).unwrap_or("/tmp/out.ppm".into());

    let mut scene = parse_scene(&input);
    render(&mut scene);
    scene.image.write(&output);
}
