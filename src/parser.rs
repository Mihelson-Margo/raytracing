use glm::{vec3, Vec3};
use itertools::izip;
use na::{Matrix3, UnitQuaternion};
use rand::rngs::ThreadRng;
use std::fs::File;
use std::io::{BufRead, BufReader};

use crate::camera::Camera;
use crate::image::*;
use crate::objects::*;

pub struct Scene {
    pub ray_depth: usize,
    pub n_samples: usize,

    pub image: Image,
    pub background_color: Vec3,
    pub camera: Camera,

    pub objects: Vec<Object<Box<dyn Geometry>>>,
    pub lights: Vec<Box<dyn LightSource>>,

    pub generator: ThreadRng,
}

#[derive(Default)]
pub struct SceneParser {
    image_width: Option<usize>,
    image_height: Option<usize>,
    background_color: Option<Vec3>,

    camera_position: Option<Vec3>,
    camera_axis: [Option<Vec3>; 3],
    camera_fov_x: Option<f32>,

    objects: Vec<Object<Box<dyn Geometry>>>,
    figure_types: Vec<FigureType>,
    // mb_lights: Vec<(Box<dyn LightSource>, usize)>,
    ray_depth: Option<usize>,
    n_samples: Option<usize>,
}

enum FigureType {
    Plane(Vec3),
    Parallelipiped(Vec3),
    Ellipsoid(Vec3),
}

impl SceneParser {
    pub fn create_scene(self) -> Scene {
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

        let lights = izip!(self.figure_types.into_iter(), self.objects.iter())
            .filter_map(|(fig_type, obj)| {
                if glm::length2(&obj.emission) == 0.0 {
                    return None;
                }
                match fig_type {
                    FigureType::Plane(_) => None,
                    FigureType::Ellipsoid(radiuses) => Some(Box::new(PositionedFigure {
                        figure: Ellipsoid { radiuses },
                        position: obj.geometry.position,
                        rotation: obj.geometry.rotation,
                    })
                        as Box<dyn LightSource>),
                    FigureType::Parallelipiped(sizes) => Some(Box::new(PositionedFigure {
                        figure: Parallelipiped { sizes },
                        position: obj.geometry.position,
                        rotation: obj.geometry.rotation,
                    })),
                }
            })
            .collect::<Vec<_>>();

        Scene {
            ray_depth: self.ray_depth.unwrap(),
            n_samples: self.n_samples.unwrap(),
            image,
            background_color: self.background_color.unwrap(),
            camera,
            objects: self.objects,
            lights,
            generator: rand::thread_rng(),
        }
    }
}

pub fn parse_scene(path: &str) -> Scene {
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
                parser.objects.push(Object::new(Box::new(Plane { normal })));
                parser.figure_types.push(FigureType::Plane(normal));
            }
            "ELLIPSOID" => {
                let radiuses = parse_vec3(&tokens[1..]);
                parser
                    .objects
                    .push(Object::new(Box::new(Ellipsoid { radiuses })));
                parser.figure_types.push(FigureType::Ellipsoid(radiuses));
            }
            "BOX" => {
                let sizes = parse_vec3(&tokens[1..]);
                parser
                    .objects
                    .push(Object::new(Box::new(Parallelipiped { sizes })));
                parser.figure_types.push(FigureType::Parallelipiped(sizes));
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
