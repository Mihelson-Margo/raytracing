use glm::{length2, vec3, Vec3};
use na::{Matrix3, UnitQuaternion};
use rand::rngs::ThreadRng;
use std::fs::File;
use std::io::{BufRead, BufReader};

use crate::bvh::Bvh;
use crate::camera::Camera;
use crate::image::*;
use crate::objects::*;

pub struct Scene {
    pub ray_depth: usize,
    pub n_samples: usize,

    // pub image: Image,
    pub background_color: Vec3,
    pub camera: Camera,

    // pub objects: Vec<Object>,
    // pub lights: Vec<PositionedFigure>,
    pub bvh: Bvh<Object>,
    pub lights_bvh: Bvh<PositionedFigure>,
}

#[derive(Default)]
pub struct SceneParser {
    image_width: Option<usize>,
    image_height: Option<usize>,
    background_color: Option<Vec3>,

    camera_position: Option<Vec3>,
    camera_axis: [Option<Vec3>; 3],
    camera_fov_x: Option<f32>,

    objects: Vec<Object>,
    ray_depth: Option<usize>,
    n_samples: Option<usize>,
}

impl SceneParser {
    pub fn create_scene(self) -> (Scene, Image) {
        let image = Image::new(self.image_width.unwrap(), self.image_height.unwrap());

        let tg_fov_x = (self.camera_fov_x.unwrap() / 2.0).tan();
        let aspect = image.height as f32 / image.width as f32;
        let tg_fov_y = aspect * tg_fov_x;
        let axis = self
            .camera_axis
            .into_iter()
            .map(Option::unwrap)
            .collect::<Vec<_>>();

        let camera = Camera {
            position: self.camera_position.unwrap(),
            axis: Matrix3::from_columns(&axis),
            tg_fov_x,
            tg_fov_y,
        };

        let lights = self
            .objects
            .iter()
            .filter_map(|obj| {
                if glm::length2(&obj.emission) == 0.0 {
                    return None;
                }
                match obj.geometry.figure {
                    Figure::Plane(_) => None,
                    _ => Some(obj.geometry.clone()),
                }
            })
            .collect::<Vec<_>>();

        let bvh = Bvh::new(self.objects);
        bvh.print();
        let lights_bvh = Bvh::new(lights);
        println!("==== Lights BVH ====");
        lights_bvh.print();
        // println!("{} lights", lights.len());

        (
            Scene {
                ray_depth: self.ray_depth.unwrap(),
                n_samples: self.n_samples.unwrap(),
                // image,
                background_color: self.background_color.unwrap(),
                camera,
                // objects: self.objects,
                bvh,
                lights_bvh,
            },
            image,
        )
    }
}

pub fn parse_scene(path: &str) -> (Scene, Image) {
    let mut parser = SceneParser::default();

    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);
    for line in reader.lines() {
        let tokens = line.as_ref().unwrap().split(' ').collect::<Vec<_>>();

        match tokens[0] {
            "DIMENSIONS" => {
                parser.image_width = Some(tokens[1].parse::<usize>().unwrap());
                parser.image_height = Some(tokens[2].parse::<usize>().unwrap());
            }
            "RAY_DEPTH" => {
                parser.ray_depth = Some(tokens[1].parse::<usize>().unwrap());
            }
            "SAMPLES" => {
                parser.n_samples = Some(tokens[1].parse::<usize>().unwrap());
            }
            "BG_COLOR" => parser.background_color = Some(parse_vec3(&tokens[1..])),
            "CAMERA_POSITION" => {
                parser.camera_position = Some(parse_vec3(&tokens[1..]));
            }
            "CAMERA_RIGHT" => {
                parser.camera_axis[0] = Some(parse_vec3(&tokens[1..]));
            }
            "CAMERA_UP" => {
                parser.camera_axis[1] = Some(parse_vec3(&tokens[1..]));
            }
            "CAMERA_FORWARD" => {
                parser.camera_axis[2] = Some(parse_vec3(&tokens[1..]));
            }
            "CAMERA_FOV_X" => {
                parser.camera_fov_x = Some(tokens[1].parse::<f32>().unwrap());
            }
            "NEW_PRIMITIVE" => {}
            "PLANE" => {
                let normal = parse_vec3(&tokens[1..]);
                parser
                    .objects
                    .push(Object::new(Figure::Plane(Plane { normal })));
            }
            "ELLIPSOID" => {
                let radiuses = parse_vec3(&tokens[1..]);
                parser
                    .objects
                    .push(Object::new(Figure::Ellipsoid(Ellipsoid { radiuses })));
            }
            "BOX" => {
                let sizes = parse_vec3(&tokens[1..]);
                parser
                    .objects
                    .push(Object::new(Figure::Parallelipiped(Parallelipiped {
                        sizes,
                    })));
            }
            "TRIANGLE" => {
                let v1 = parse_vec3(&tokens[1..=3]);
                let v2 = parse_vec3(&tokens[4..=6]);
                let v3 = parse_vec3(&tokens[7..=9]);
                parser
                    .objects
                    .push(Object::new(Figure::Triangle(Triangle::new(v1, v2, v3))));
            }
            "POSITION" => {
                let position = parse_vec3(&tokens[1..]);
                let idx = parser.objects.len() - 1;
                parser.objects[idx].geometry.position = position;
            }
            "ROTATION" => {
                let rotation = parse_quaternion(&tokens[1..]);
                let idx = parser.objects.len() - 1;
                parser.objects[idx].geometry.rotation = rotation;
            }
            "COLOR" => {
                let color = parse_vec3(&tokens[1..]);
                let idx = parser.objects.len() - 1;
                parser.objects[idx].color = color;
            }
            "EMISSION" => {
                let color = parse_vec3(&tokens[1..]);
                let idx = parser.objects.len() - 1;
                parser.objects[idx].emission = color;
            }
            "METALLIC" => {
                let idx = parser.objects.len() - 1;
                parser.objects[idx].material = Material::Metallic;
            }
            "DIELECTRIC" => {
                let idx = parser.objects.len() - 1;
                parser.objects[idx].material = Material::Dielectric { ior: 1.0 };
            }
            "IOR" => {
                let ior = tokens[1].parse::<f32>().unwrap();
                let idx = parser.objects.len() - 1;
                if let Material::Dielectric { .. } = parser.objects[idx].material {
                    parser.objects[idx].material = Material::Dielectric { ior };
                }
            }
            _ => {}
        }
    }

    parser.create_scene()
}

fn parse_vec3(tokens: &[&str]) -> Vec3 {
    let r = tokens[0].parse::<f32>().unwrap();
    let g = tokens[1].parse::<f32>().unwrap();
    let b = tokens[2].parse::<f32>().unwrap();

    vec3(r, g, b)
}

fn parse_quaternion(tokens: &[&str]) -> UnitQuaternion<f32> {
    let x = tokens[0].parse::<f32>().unwrap();
    let y = tokens[1].parse::<f32>().unwrap();
    let z = tokens[2].parse::<f32>().unwrap();
    let w = tokens[3].parse::<f32>().unwrap();

    let q = na::Quaternion::<f32>::new(w, x, y, z);
    UnitQuaternion::from_quaternion(q)
}
