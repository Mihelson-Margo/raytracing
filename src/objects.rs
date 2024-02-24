use glm::Vec3;
use itertools::MultiUnzip;
use na::UnitQuaternion;

use crate::ray::Ray;

pub struct RayIntersection {
    pub t: f32,
    pub n: Vec3,
    pub is_inside: bool,
}

pub trait Geometry {
    fn intersect(&self, ray: &Ray) -> Option<RayIntersection>;
}

pub enum Material {
    Diffuse,
    Metallic,
    Dielectric { ior: f32 },
}

pub struct Object<G> {
    pub geometry: G,

    pub position: Vec3,
    pub rotation: UnitQuaternion<f32>,

    pub color: Vec3,
    pub material: Material,
}

impl<G> Object<G> {
    pub fn new(geometry: G) -> Self {
        Self {
            geometry,
            position: Vec3::zeros(),
            rotation: UnitQuaternion::identity(),
            color: Vec3::zeros(),
            material: Material::Diffuse,
        }
    }
}

impl Object<Box<dyn Geometry>> {
    pub fn intersect(&self, ray: &Ray) -> Option<RayIntersection> {
        let transformed_ray = Ray {
            origin: self.rotation.inverse() * (ray.origin - self.position),
            direction: self.rotation.inverse() * ray.direction,
        };
        let mut intersection = self.geometry.intersect(&transformed_ray)?;

        intersection.n = (self.rotation * intersection.n).normalize();
        if glm::dot(&intersection.n, &ray.direction) > 0.0 {
            intersection.n = -intersection.n;
        }

        Some(intersection)
    }
}

pub struct Plane {
    // contains 0
    pub normal: Vec3,
}

impl Geometry for Plane {
    fn intersect(&self, ray: &Ray) -> Option<RayIntersection> {
        let t = -glm::dot(&ray.origin, &self.normal) / glm::dot(&ray.direction, &self.normal);
        let is_inside = glm::dot(&self.normal, &ray.origin) < 0.0;

        if t < 0.0 {
            None
        } else {
            Some(RayIntersection {
                t,
                n: self.normal,
                is_inside,
            })
        }
    }
}

pub struct Ellipsoid {
    // center is 0
    pub radiuses: Vec3,
}

impl Geometry for Ellipsoid {
    fn intersect(&self, ray: &Ray) -> Option<RayIntersection> {
        let u = ray.origin.component_div(&self.radiuses);
        let v = ray.direction.component_div(&self.radiuses);

        let a = glm::length2(&v);
        let b = glm::dot(&u, &v);
        let c = glm::length2(&u) - 1.0;

        let det = b * b - a * c;

        if det < 0.0 {
            return None;
        }

        let t1 = (-b + det.sqrt()) / a;
        let t2 = (-b - det.sqrt()) / a;

        let (t1, t2) = (t1.min(t2), t1.max(t2));

        let t = if t1 > 0.0 {
            Some(t1)
        } else if t2 > 0.0 {
            Some(t2)
        } else {
            None
        }?;

        Some(RayIntersection {
            t,
            is_inside: glm::length2(&u) < 1.0,
            n: (u + t * v).component_div(&self.radiuses),
        })
    }
}

pub struct Parallelipiped {
    // center is 0
    pub sizes: Vec3,
}

impl Geometry for Parallelipiped {
    fn intersect(&self, ray: &Ray) -> Option<RayIntersection> {
        let o = ray.origin;
        let d = ray.direction;

        let (l, r): (Vec<_>, Vec<_>) = (0..3)
            .map(|i| {
                let t1 = (self.sizes[i] - o[i]) / d[i];
                let t2 = (-self.sizes[i] - o[i]) / d[i];

                (t1.min(t2), t1.max(t2))
            })
            .multiunzip();

        let t1 = l[0].max(l[1]).max(l[2]);
        let t2 = r[0].min(r[1]).min(r[2]);

        let t = if t1 > t2 {
            None
        } else if t1 >= 0.0 {
            Some(t1)
        } else if t2 >= 0.0 {
            Some(t2)
        } else {
            None
        }?;

        let mut n = (o + t * d).component_div(&self.sizes);
        let (i, _) = n.abs().argmax();
        n[(i + 1) % 3] = 0.0;
        n[(i + 2) % 3] = 0.0;

        Some(RayIntersection {
            t,
            is_inside: o.component_div(&self.sizes).abs().max() < 1.0,
            n,
        })
    }
}
