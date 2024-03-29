use std::f32::consts::PI;

use glm::Vec3;
use rand::Rng;

use crate::objects::{Geometry, Material, Object, RayIntersection};
use crate::random::{ToLight, MIS};
use crate::ray::Ray;
use crate::Scene;

pub fn trace_ray(scene: &mut Scene, ray: &Ray, depth: usize) -> Vec3 {
    if depth >= scene.ray_depth {
        return Vec3::zeros();
    }

    let Some((idx, intersection)) = intersect_with_objects(&scene.objects, ray, f32::INFINITY)
    else {
        return scene.background_color;
    };

    let point = ray.origin + intersection.t * ray.direction;
    let normal = intersection.n;
    let emitted = scene.objects[idx].emission;

    let color = match scene.objects[idx].material {
        Material::Diffuse => {
            let color_obj = scene.objects[idx].color / PI;

            let distribution = MIS {
                to_light: ToLight {
                    lights: &scene.lights,
                },
            };

            let new_dir = distribution.sample(&point, &normal, &mut scene.generator);
            if glm::dot(&new_dir, &normal) < 0.0 {
                Vec3::zeros()
            } else {
                let pdf = distribution.pdf(&point, &normal, &new_dir);
                if !pdf.is_finite() || pdf < 1e-6 {
                    Vec3::zeros()
                } else {
                    let new_ray = Ray::new_shifted(point, new_dir);
                    let cos = glm::dot(&normal, &new_ray.direction);

                    let color_in = trace_ray(scene, &new_ray, depth + 1);

                    color_in.component_mul(&color_obj) * cos / pdf
                }
            }
        }
        Material::Metallic => {
            let reflected_ray = get_reflected_ray(&ray.direction, &point, &normal);
            let color = trace_ray(scene, &reflected_ray, depth + 1);
            color.component_mul(&scene.objects[idx].color)
        }
        Material::Dielectric { ior } => calc_dielectric_color(
            scene,
            ray,
            &point,
            &normal,
            intersection.is_inside,
            ior,
            idx,
            depth,
        ),
    };

    color + emitted
}

fn calc_dielectric_color(
    scene: &mut Scene,
    ray: &Ray,
    point: &Vec3,
    normal: &Vec3,
    is_inside: bool,
    ior: f32,
    object_idx: usize,
    depth: usize,
) -> Vec3 {
    // eta = eta_from / eta_to
    let eta = if is_inside { ior } else { 1.0 / ior };

    let reflected_ray = get_reflected_ray(&ray.direction, point, normal);
    let maybe_refracetd_ray = get_refracted_ray(&ray.direction, point, normal, eta);
    let coeff = schilcks_coeff(eta, -glm::dot(&ray.direction, normal));

    if maybe_refracetd_ray.is_some() && (scene.generator.gen::<f32>() < 1.0 - coeff) {
        let refracted_ray = maybe_refracetd_ray.unwrap();
        let mut color = trace_ray(scene, &refracted_ray, depth + 1);
        if !is_inside {
            color.component_mul_assign(&scene.objects[object_idx].color);
        }
        color
    } else {
        trace_ray(scene, &reflected_ray, depth + 1)
    }
}

fn intersect_with_objects(
    objects: &[Object],
    ray: &Ray,
    max_dist: f32,
) -> Option<(usize, RayIntersection)> {
    let ray_length = glm::length(&ray.direction);

    objects
        .iter()
        .enumerate()
        .filter_map(|(i, object)| object.geometry.intersect(ray).map(|res| (i, res)))
        .filter_map(|(i, res)| {
            if res.t * ray_length < max_dist {
                Some((i, res))
            } else {
                None
            }
        })
        .min_by(|(_, a), (_, b)| a.t.partial_cmp(&b.t).unwrap())
}

fn get_reflected_ray(direction: &Vec3, point: &Vec3, normal: &Vec3) -> Ray {
    let new_dir = direction - 2.0 * normal * glm::dot(direction, normal);
    Ray::new_shifted(*point, new_dir)
}

fn get_refracted_ray(direction: &Vec3, point: &Vec3, normal: &Vec3, eta: f32) -> Option<Ray> {
    assert!((glm::length2(normal) - 1.0) < 1e-5);
    assert!((glm::length2(direction) - 1.0) < 1e-5);

    let cos1 = -glm::dot(normal, direction);
    let sin2 = eta * (1.0 - cos1 * cos1).sqrt();

    if sin2.abs() > 1.0 {
        return None;
    }

    let cos2 = (1.0 - sin2 * sin2).sqrt();
    let new_dir = eta * direction + (eta * cos1 - cos2) * normal;
    Some(Ray::new_shifted(*point, new_dir))
}

fn schilcks_coeff(eta: f32, cos: f32) -> f32 {
    let r0 = (eta - 1.0) / (eta + 1.0);
    let r0 = r0 * r0;

    r0 + (1.0 - r0) * (1.0 - cos).powi(5)
}
