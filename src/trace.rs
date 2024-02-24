use glm::Vec3;

use crate::objects::{Geometry, Material, Object, RayIntersection};
use crate::ray::Ray;
use crate::Scene;

pub fn trace_ray(scene: &Scene, ray: &Ray, depth: usize) -> Option<Vec3> {
    if depth >= scene.ray_depth {
        return None;
    }

    let (idx, intersection) = intersect_with_objects(&scene.objects, ray, f32::INFINITY)?;
    let point = ray.origin + intersection.t * ray.direction;

    let color = match scene.objects[idx].material {
        Material::Diffuse => calc_diffuse_color(scene, &point, &intersection.n, idx),
        Material::Metallic => {
            let reflected_ray = get_reflected_ray(&ray.direction, &point, &intersection.n);
            let color =
                trace_ray(scene, &reflected_ray, depth + 1).unwrap_or(scene.background_color);
            color.component_mul(&scene.objects[idx].color)
        }
        Material::Dielectric { ior } => calc_dielectric_color(
            scene,
            ray,
            &point,
            &intersection.n,
            intersection.is_inside,
            ior,
            idx,
            depth,
        ),
    };

    Some(color)
}

fn calc_diffuse_color(scene: &Scene, point: &Vec3, normal: &Vec3, object_idx: usize) -> Vec3 {
    let mut light_intensity = scene.ambient;
    for light_source in &scene.lights {
        let d = light_source.direction_to_light(point).normalize(); // TODO: fix
        let ray_to_light = Ray::new_shifted(*point, d);

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

    light_intensity.component_mul(&scene.objects[object_idx].color)
}

fn calc_dielectric_color(
    scene: &Scene,
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
    let reflected_color =
        trace_ray(scene, &reflected_ray, depth + 1).unwrap_or(scene.background_color);

    let maybe_refracetd_ray = get_refracted_ray(&ray.direction, point, normal, eta);
    let (refracted_color, coeff) = if let Some(refracted_ray) = maybe_refracetd_ray {
        let mut color =
            trace_ray(scene, &refracted_ray, depth + 1).unwrap_or(scene.background_color);
        if !is_inside {
            color.component_mul_assign(&scene.objects[object_idx].color);
        }
        let cos = -glm::dot(&ray.direction, normal);
        (color, schilcks_coeff(eta, cos))
    } else {
        (Vec3::zeros(), 1.0)
    };

    reflected_color * coeff + refracted_color * (1.0 - coeff)
}

fn intersect_with_objects(
    objects: &[Object<Box<dyn Geometry>>],
    ray: &Ray,
    max_dist: f32,
) -> Option<(usize, RayIntersection)> {
    let ray_length = glm::length(&ray.direction);

    objects
        .iter()
        .enumerate()
        .filter_map(|(i, object)| object.intersect(ray).map(|res| (i, res)))
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
