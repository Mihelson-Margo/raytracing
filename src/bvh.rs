use itertools::partition;
use noisy_float::types::R32;

use crate::{
    objects::{Aabb, Geometry, RayIntersection},
    ray::Ray,
};

const EPS: f32 = 1e-4;

#[derive(Debug)]
struct Node {
    aabb: Aabb,
    children: Option<(usize, usize)>,
    first_obj_idx: usize,
    last_obj_idx: usize,
}

pub struct Bvh<G> {
    nodes: Vec<Node>,
    root: usize,
    pub objects: Vec<G>,
    leaves_cnt: usize,
}

// type Callback = Fn (usize, usize) -> f32;

impl<G: Geometry> Bvh<G> {
    pub fn new(mut objects: Vec<G>) -> Self {
        let last_root_idx = partition(&mut objects, |obj| {
            let aabb = obj.calc_aabb();
            let size = glm::length2(&(aabb.max - aabb.min));
            size.is_infinite()
        });
        let root = Node {
            aabb: Aabb::empty(),
            children: None,
            first_obj_idx: 0,
            last_obj_idx: last_root_idx,
        };

        let mut bvh = Bvh {
            nodes: vec![root],
            root: 0,
            objects,
            leaves_cnt: 0,
        };

        bvh.build_node(0, last_root_idx, bvh.objects.len());
        bvh.nodes[bvh.root].first_obj_idx = 0;
        bvh
    }

    pub fn intersect(&self, ray: &Ray) -> Option<(usize, RayIntersection)> {
        self.intersect_node(self.root, ray, None)
    }

    pub fn intersect_all<F>(&self, ray: &Ray, callback: &F) -> f32
    where
        F: Fn(&G, &Ray, &RayIntersection) -> f32,
    {
        self.intersect_all_in_node(self.root, ray, callback)
    }

    pub fn get_object(&self, object_idx: usize) -> &'_ G {
        &self.objects[object_idx]
    }

    pub fn get_n_objects(&self) -> usize {
        self.objects.len()
    }

    fn build_node(&mut self, node_idx: usize, from_idx: usize, to_idx: usize) {
        let node = &mut self.nodes[node_idx];
        let mut aabbs = self.objects[from_idx..to_idx]
            .iter()
            .map(|obj| obj.calc_aabb())
            .collect::<Vec<_>>();
        node.aabb = Aabb::empty();
        for obj_aabb in aabbs.iter() {
            node.aabb.extend(obj_aabb);
        }

        if to_idx - from_idx <= 4 {
            // TODO: fix this hack
            // node.last_obj_idx = node.last_obj_idx.max(to_idx);
            mark_terminal_node(node, from_idx, to_idx);
            self.leaves_cnt += 1;
            return;
        }

        // let (axis, _) = (node.aabb.max - node.aabb.min).argmax();
        // let split_value = node.aabb.max[axis] + node.aabb.min[axis];

        let mut best_split = (0, f32::INFINITY);
        let mut best_quality = node.aabb.area() * (to_idx - from_idx) as f32;
        for axis in 0..3 {
            aabbs.sort_by_key(|a| R32::try_new(a.min[axis] + a.max[axis]).unwrap());
            let mut prefix_areas = vec![0.0; aabbs.len() + 1];
            let mut suffix_areas = vec![0.0; aabbs.len() + 1];
            let mut prefix_aabb = Aabb::empty();
            let mut suffix_aabb = Aabb::empty();
            for i in 0..aabbs.len() {
                prefix_aabb.extend(&aabbs[i]);
                prefix_areas[i + 1] = prefix_aabb.area();
                let j = aabbs.len() - i - 1;
                suffix_aabb.extend(&aabbs[j]);
                suffix_areas[j] = suffix_aabb.area();
            }

            for i in 0..aabbs.len() {
                let quality =
                    prefix_areas[i] * i as f32 + suffix_areas[i] * (aabbs.len() - i) as f32;
                if quality < best_quality {
                    best_quality = quality;
                    best_split = (axis, aabbs[i].max[axis] + aabbs[i].min[axis]);
                }
            }
        }

        // println!("q {}, {:?}", best_quality, best_split);

        let split_axis = best_split.0;
        let split_value = best_split.1;
        let mut split_idx = partition(&mut self.objects[from_idx..to_idx], |obj| {
            let aabb = obj.calc_aabb();
            (aabb.max[split_axis] + aabb.min[split_axis]) < split_value
        });

        if split_idx == 0 || split_idx == to_idx - from_idx {
            // node.last_obj_idx = node.last_obj_idx.max(to_idx);
            mark_terminal_node(node, from_idx, to_idx);
            self.leaves_cnt += 1;
            return;
        }
        split_idx += from_idx;

        let left = Node {
            aabb: Aabb::empty(),
            children: None,
            first_obj_idx: 0,
            last_obj_idx: 0,
        };
        let right = Node {
            aabb: Aabb::empty(),
            children: None,
            first_obj_idx: 0,
            last_obj_idx: 0,
        };
        let left_node_idx = self.nodes.len();
        self.nodes.push(left);
        self.nodes.push(right);
        self.nodes[node_idx].children = Some((left_node_idx, left_node_idx + 1));
        self.build_node(left_node_idx, from_idx, split_idx);
        self.build_node(left_node_idx + 1, split_idx, to_idx);
    }

    fn intersect_node(
        &self,
        node_idx: usize,
        ray: &Ray,
        best_intersection: Option<(usize, RayIntersection)>,
    ) -> Option<(usize, RayIntersection)> {
        let node = &self.nodes[node_idx];

        let mut intersection = best_intersection;
        for idx in node.first_obj_idx..node.last_obj_idx {
            let obj = &self.objects[idx];
            let new_intersection = obj.intersect(ray).map(|i| (idx, i));
            if is_closer(&new_intersection, &intersection) {
                intersection = new_intersection;
            }
        }

        let Some((mut left, mut right)) = node.children else {
            return intersection;
        };

        let mut left_i = self.nodes[left].aabb.intersect(ray);
        let mut right_i = self.nodes[right].aabb.intersect(ray);
        if right_i < left_i {
            (left, right) = (right, left);
            (left_i, right_i) = (right_i, left_i);
        }

        if left_i.is_infinite()
            || intersection
                .as_ref()
                .map(|(_, i)| i.t)
                .unwrap_or(f32::INFINITY)
                < left_i
        {
            return intersection;
        }

        let new_intersection = self.intersect_node(left, ray, intersection.clone());
        if is_closer(&new_intersection, &intersection) {
            intersection = new_intersection;
        }

        if right_i.is_infinite()
            || intersection
                .as_ref()
                .map(|(_, i)| i.t)
                .unwrap_or(f32::INFINITY)
                < right_i
        {
            return intersection;
        }
        let new_intersection = self.intersect_node(right, ray, intersection.clone());
        if is_closer(&new_intersection, &intersection) {
            intersection = new_intersection;
        }

        intersection
    }

    fn intersect_all_in_node<F>(&self, node_idx: usize, ray: &Ray, callback: &F) -> f32
    where
        F: Fn(&G, &Ray, &RayIntersection) -> f32,
    {
        let mut result = 0.0;
        let node = &self.nodes[node_idx];

        for idx in node.first_obj_idx..node.last_obj_idx {
            let obj = &self.objects[idx];
            if let Some(intersection) = obj.intersect(ray) {
                result += callback(obj, ray, &intersection);

                let shifted_ray = Ray {
                    origin: ray.origin + (intersection.t + EPS) * ray.direction,
                    direction: ray.direction,
                };
                if let Some(mut intersection2) = obj.intersect(&shifted_ray) {
                    intersection2.t += intersection.t + EPS;
                    result += callback(obj, ray, &intersection2);
                }
            }
        }

        let Some((left, right)) = node.children else {
            return result;
        };

        if self.nodes[left].aabb.intersect(ray).is_finite() {
            result += self.intersect_all_in_node(left, ray, callback);
        }

        if self.nodes[right].aabb.intersect(ray).is_finite() {
            result += self.intersect_all_in_node(right, ray, callback);
        }

        result
    }

    pub fn print(&self) {
        println!(
            "Bhv has {} nodes, {} leaves and {} objects",
            self.nodes.len(),
            self.leaves_cnt,
            self.objects.len()
        );
        println!("Depth: {}", self.dfs(self.root));

        // println!("Root: {:?}", self.nodes[self.root]);

        // for (i, node) in self.nodes.iter().enumerate() {
        //     println!("{}) {:?}", i, node);
        // }
    }

    fn dfs(&self, node_idx: usize) -> usize {
        let Some((l, r)) = self.nodes[node_idx].children else {
            return 1;
        };
        self.dfs(l).max(self.dfs(r)) + 1
    }

    pub fn check_bvh(&self) {
        let mut mask = vec![false; self.objects.len()];
        for node in &self.nodes {
            for idx in node.first_obj_idx..node.last_obj_idx {
                assert!(!mask[idx]);
                mask[idx] = true;
            }

            if let Some((l, r)) = node.children {
                assert!(node.aabb.contains(&self.nodes[l].aabb));
                assert!(node.aabb.contains(&self.nodes[r].aabb));
            }
        }

        for val in &mask {
            assert!(val);
        }

        println!("OK!");
        // assert!(false);
    }
}

fn is_closer(
    one: &Option<(usize, RayIntersection)>,
    other: &Option<(usize, RayIntersection)>,
) -> bool {
    let Some(t) = one.as_ref().map(|i| i.1.t) else {
        return false;
    };
    let t_other = other.as_ref().map(|i| i.1.t).unwrap_or(f32::INFINITY);
    t < t_other
}

fn mark_terminal_node(node: &mut Node, first_idx: usize, last_idx: usize) {
    node.children = None;
    node.first_obj_idx = first_idx;
    node.last_obj_idx = last_idx;
}
