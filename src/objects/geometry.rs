use std::mem::swap;

use glm::{vec3, Vec3};
use itertools::MultiUnzip;
use na::Matrix3;

use super::{
    figures::{Ellipsoid, Parallelipiped, Plane},
    Figure, PositionedFigure, Triangle,
};
use crate::ray::Ray;

#[derive(Clone, Debug)]
pub struct RayIntersection {
    pub t: f32,
    pub n: Vec3,
    pub is_inside: bool,
}

#[derive(Debug)]
pub struct Aabb {
    pub min: Vec3,
    pub max: Vec3,

    pub sizes: Vec3,
    pub center: Vec3,
}

impl Aabb {
    pub fn empty() -> Self {
        Aabb {
            min: vec3(f32::INFINITY, f32::INFINITY, f32::INFINITY),
            max: vec3(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY),
            sizes: vec3(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY),
            center: Vec3::zeros(),
        }
    }

    pub fn add_point(&mut self, p: Vec3) {
        for i in 0..3 {
            self.min[i] = self.min[i].min(p[i]);
            self.max[i] = self.max[i].max(p[i]);
        }
        self.update();
    }

    pub fn extend(&mut self, other: &Aabb) {
        for i in 0..3 {
            self.min[i] = self.min[i].min(other.min[i]);
            self.max[i] = self.max[i].max(other.max[i]);
        }
        self.update();
    }

    pub fn area(&self) -> f32 {
        let sizes = self.max - self.min;
        sizes.x * sizes.y + sizes.y * sizes.z + sizes.z * sizes.x
    }

    fn update(&mut self) {
        self.center = (self.min + self.max) / 2.0;
        self.sizes = (self.max - self.min) / 2.0;
    }

    pub fn intersect(&self, ray: &Ray) -> f32 {
        let o = ray.origin - self.center;
        let d = ray.direction;

        let mut l = f32::NEG_INFINITY;
        let mut r = f32::INFINITY;
        for i in 0..3 {
            let a = (self.sizes[i] - o[i]) / d[i];
            let b = -(self.sizes[i] + o[i]) / d[i];
            l = l.max(a.min(b));
            r = r.min(a.max(b));
        }

        if l > r {
            f32::INFINITY
        } else {
            l
        }
    }

    pub fn contains(&self, other: &Aabb) -> bool {
        (self.min <= other.min) && (other.max <= self.max)
    }
}

pub trait Geometry {
    fn intersect(&self, ray: &Ray) -> Option<RayIntersection>;
    fn calc_aabb(&self) -> Aabb;
}

impl Geometry for PositionedFigure {
    fn intersect(&self, ray: &Ray) -> Option<RayIntersection> {
        let transformed_ray = Ray {
            origin: self.rotation_inv * (ray.origin - self.position),
            direction: self.rotation_inv * ray.direction,
        };
        let mut intersection = self.figure.intersect(&transformed_ray)?;

        intersection.n = (self.rotation * intersection.n).normalize();
        if glm::dot(&intersection.n, &ray.direction) > 0.0 {
            intersection.n = -intersection.n;
        }

        Some(intersection)
    }

    fn calc_aabb(&self) -> Aabb {
        let initial_aabb = self.figure.calc_aabb();
        let mut new_aabb = Aabb::empty();
        for v in cube_vertices(&initial_aabb.min, &initial_aabb.max) {
            new_aabb.add_point(self.rotation * v + self.position);
        }
        new_aabb
    }
}

impl Geometry for Figure {
    fn intersect(&self, ray: &Ray) -> Option<RayIntersection> {
        match &self {
            Figure::Plane(plane) => plane.intersect(ray),
            Figure::Ellipsoid(ellipsoid) => ellipsoid.intersect(ray),
            Figure::Parallelipiped(parallelipiped) => parallelipiped.intersect(ray),
            Figure::Triangle(triangle) => triangle.intersect(ray),
        }
    }

    fn calc_aabb(&self) -> Aabb {
        match &self {
            Figure::Plane(plane) => plane.calc_aabb(),
            Figure::Ellipsoid(ellipsoid) => ellipsoid.calc_aabb(),
            Figure::Parallelipiped(parallelipiped) => parallelipiped.calc_aabb(),
            Figure::Triangle(triangle) => triangle.calc_aabb(),
        }
    }
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

    fn calc_aabb(&self) -> Aabb {
        Aabb {
            min: vec3(f32::NEG_INFINITY, f32::NEG_INFINITY, f32::NEG_INFINITY),
            max: vec3(f32::INFINITY, f32::INFINITY, f32::INFINITY),
            sizes: vec3(f32::INFINITY, f32::INFINITY, f32::INFINITY),
            center: Vec3::zeros(),
        }
    }
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

    fn calc_aabb(&self) -> Aabb {
        let mut aabb = Aabb::empty();
        for v in cube_vertices(&-self.radiuses, &self.radiuses) {
            aabb.add_point(v);
        }
        aabb
    }
}

impl Parallelipiped {
    fn intersect_checked(&self, ray: &Ray, check_neg_t: bool) -> Option<RayIntersection> {
        let o = ray.origin;
        let d = ray.direction;

        let mut l = (self.sizes - o).component_div(&d);
        let mut r = (-self.sizes - o).component_div(&d);
        for i in 0..3 {
            if l[i] > r[i] {
                swap(&mut l[i], &mut r[i]);
            }
        }

        let t1 = l[0].max(l[1]).max(l[2]);
        let t2 = r[0].min(r[1]).min(r[2]);

        let t = if t1 > t2 {
            None
        } else if (t1 >= 0.0) || (!check_neg_t) {
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

impl Geometry for Parallelipiped {
    fn intersect(&self, ray: &Ray) -> Option<RayIntersection> {
        self.intersect_checked(ray, true)
    }

    fn calc_aabb(&self) -> Aabb {
        let mut aabb = Aabb::empty();
        for v in cube_vertices(&-self.sizes, &self.sizes) {
            aabb.add_point(v);
        }
        aabb
    }
}

impl Geometry for Triangle {
    fn intersect(&self, ray: &Ray) -> Option<RayIntersection> {
        // TODO: fix
        let mat = Matrix3::from_columns(&[self.edge1, self.edge2, -ray.direction]);
        let Some(mat_inv) = mat.try_inverse() else {
            return None;
        };
        let res = mat_inv * (ray.origin - self.v);
        let u = res.x;
        let v = res.y;
        let t = res.z;

        let is_inside = glm::dot(&self.normal, &ray.origin) < 0.0;

        if t < 0.0 || u < 0.0 || v < 0.0 || u + v > 1.0 {
            None
        } else {
            Some(RayIntersection {
                t,
                n: self.normal,
                is_inside,
            })
        }
    }

    fn calc_aabb(&self) -> Aabb {
        let mut aabb = Aabb::empty();
        aabb.add_point(self.v);
        aabb.add_point(self.v + self.edge1);
        aabb.add_point(self.v + self.edge2);
        aabb
    }
}

fn cube_vertices(min_vertex: &Vec3, max_vertex: &Vec3) -> Vec<Vec3> {
    (0..8)
        .map(|i| {
            Vec3::from_iterator((0..3).map(|j| {
                if (i / (1 << j)) % 2 == 0 {
                    min_vertex[j]
                } else {
                    max_vertex[j]
                }
            }))
        })
        .collect()
}
