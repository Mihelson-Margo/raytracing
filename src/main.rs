mod camera;
mod image;
mod light;
mod objects;
mod parser;

use glm::Vec3;

use objects::{Geometry, Material, Object, Ray, RayIntersection};
use parser::*;

const EPS: f32 = 1e-4;

fn intersect_with_objects(
    objects: &[Object<Box<dyn Geometry>>],
    ray: &Ray,
    max_dist: f32,
) -> Option<(usize, RayIntersection)> {
    let ray_length = glm::length(&ray.direction);

    objects
        .iter()
        .enumerate()
        .filter_map(|(i, object)| object.intersect(&ray).map(|res| (i, res)))
        .filter_map(|(i, res)| {
            if res.t * ray_length < max_dist {
                Some((i, res))
            } else {
                None
            }
        })
        .min_by(|(_, a), (_, b)| a.t.partial_cmp(&b.t).unwrap())
}

fn color_of_diffuse(scene: &Scene, point: &Vec3, normal: &Vec3, object_idx: usize) -> Option<Vec3> {
    let mut light_intensity = scene.ambient;
    for light_source in &scene.lights {
        let d = light_source.direction_to_light(point).normalize(); // TODO: fix
        let ray_to_light = Ray {
            origin: point + EPS * d,
            direction: d,
        };

        if intersect_with_objects(
            &scene.objects,
            &ray_to_light,
            light_source.dist_to_light(point),
        )
        .is_none()
        {
            let coef = glm::dot(normal, &d).max(0.0);
            light_intensity += light_source.intensity(point) * coef;
        }
    }
    let color = light_intensity.component_mul(&scene.objects[object_idx].color);
    Some(color)
}

fn reflected_ray(ray: &Ray, point: &Vec3, n: &Vec3) -> Ray {
    let d = ray.direction;
    let new_d = d - 2.0 * n * glm::dot(&d, &n);
    Ray {
        origin: point + EPS * new_d,
        direction: new_d,
    }
}

fn schilcks_coeff(eta1: f32, eta2: f32, cos_angle: f32) -> f32 {
    let r0 = (eta1 - eta2) / (eta1 + eta2);
    let r0 = r0 * r0;

    r0 + (1.0 - r0) * (1.0 - cos_angle).powi(5)
}

fn trace_ray(scene: &mut Scene, ray: &Ray, depth: usize) -> Option<Vec3> {
    if depth >= scene.ray_depth {
        return None;
    }

    let (idx, intersection) = intersect_with_objects(&scene.objects, ray, f32::INFINITY)?;

    match scene.objects[idx].material {
        Material::Metallic => {
            let point = ray.origin + intersection.t * ray.direction;
            let new_ray = reflected_ray(ray, &point, &intersection.n);
            let color = trace_ray(scene, &new_ray, depth + 1).unwrap_or(scene.background_color);
            Some(color.component_mul(&scene.objects[idx].color))
        }
        Material::Dielectric { ior } => {
            if depth == 0 {
                assert!(!intersection.is_inside);
            }
            let (eta1, eta2) = if intersection.is_inside {
                (ior, 1.0)
            } else {
                (1.0, ior)
            };
            let eta = eta1 / eta2;

            let point = ray.origin + intersection.t * ray.direction;

            let reflected_ray = reflected_ray(&ray, &point, &intersection.n);
            let reflected_color =
                trace_ray(scene, &reflected_ray, depth + 1).unwrap_or(scene.background_color);

            let n = intersection.n.normalize();
            let d = ray.direction.normalize();
            let cos1 = -glm::dot(&n, &d);
            let sin2 = eta * (1.0 - cos1 * cos1).sqrt();

            let (refraction_col, reflection_coeff) = if sin2.abs() < 1.0 {
                let cos2 = (1.0 - sin2 * sin2).sqrt();

                let new_d = eta * d + (eta * cos1 - cos2) * n;
                let new_ray = Ray {
                    origin: point + EPS * new_d,
                    direction: new_d,
                };
                let color = trace_ray(scene, &new_ray, depth + 1).unwrap_or(scene.background_color);

                (
                    color.component_mul(&scene.objects[idx].color),
                    schilcks_coeff(eta1, eta2, cos1),
                )
            } else {
                (Vec3::zeros(), 1.0)
            };

            let color =
                reflected_color * reflection_coeff + refraction_col * (1.0 - reflection_coeff);
            Some(color)
            //Some(color.component_mul(&scene.objects[idx].color))
        }
        _ => {
            let point = ray.origin + intersection.t * ray.direction;
            color_of_diffuse(scene, &point, &intersection.n, idx)
        }
    }
}

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
