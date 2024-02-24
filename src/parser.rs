use glm::{vec3, Vec3};
use na::{Matrix3, UnitQuaternion};
use std::fs::File;
use std::io::{BufRead, BufReader};

use crate::camera::Camera;
use crate::image::*;
use crate::light::{DirectedLight, Light, PointLight};
use crate::objects::*;

pub struct Scene {
    pub ray_depth: usize,

    pub image: Image,
    pub background_color: Vec3,
    pub camera: Camera,

    pub objects: Vec<Object<Box<dyn Geometry>>>,

    pub lights: Vec<Box<dyn Light>>,
    pub ambient: Vec3,
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

    ambient: Option<Vec3>,
    lights: Vec<Box<dyn Light>>,
    have_new_light: bool,
    last_intensity: Option<Vec3>,
    last_light_direction: Option<Vec3>,
    last_light_position: Option<Vec3>,
    last_attenuation: Option<Vec3>,

    ray_depth: Option<usize>,
}

impl SceneParser {
    pub fn create_scene(self) -> Scene {
        let image = Image::new(
            self.image_width.unwrap(),
            self.image_height.unwrap(),
            self.background_color.unwrap(),
        );

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

        Scene {
            ray_depth: self.ray_depth.unwrap(),
            image,
            background_color: self.background_color.unwrap(),
            camera,
            objects: self.objects,
            lights: self.lights,
            ambient: self.ambient.unwrap_or(Vec3::zeros()),
        }
    }

    fn push_light(&mut self) {
        if !self.have_new_light {
            return;
        }

        if self.last_light_direction.is_some() {
            self.lights.push(Box::new(DirectedLight {
                direction: self.last_light_direction.unwrap(),
                intensity: self.last_intensity.unwrap(),
            }));
        } else {
            self.lights.push(Box::new(PointLight {
                position: self.last_light_position.unwrap(),
                attenuation: self.last_attenuation.unwrap(),
                intensity: self.last_intensity.unwrap(),
            }));
        }

        self.last_intensity = None;
        self.last_light_direction = None;
        self.last_light_position = None;
        self.last_attenuation = None;
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
            "BG_COLOR" => parser.background_color = Some(parse_vec3(&tokens[1..])),
            "AMBIENT_LIGHT" => parser.ambient = Some(parse_vec3(&tokens[1..])),
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
            }
            "ELLIPSOID" => {
                let radiuses = parse_vec3(&tokens[1..]);
                parser
                    .objects
                    .push(Object::new(Box::new(Ellipsoid { radiuses })));
            }
            "BOX" => {
                let sizes = parse_vec3(&tokens[1..]);
                parser
                    .objects
                    .push(Object::new(Box::new(Parallelipiped { sizes })));
            }
            "POSITION" => {
                let position = parse_vec3(&tokens[1..]);
                let idx = parser.objects.len() - 1;
                parser.objects[idx].position = position;
            }
            "ROTATION" => {
                let rotation = parse_quaternion(&tokens[1..]);
                let idx = parser.objects.len() - 1;
                parser.objects[idx].rotation = rotation;
            }
            "COLOR" => {
                let color = parse_vec3(&tokens[1..]);
                let idx = parser.objects.len() - 1;
                parser.objects[idx].color = color;
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
                match parser.objects[idx].material {
                    Material::Dielectric { .. } => {
                        parser.objects[idx].material = Material::Dielectric { ior };
                    }
                    _ => {}
                };
            }
            "NEW_LIGHT" => {
                parser.push_light();
                parser.have_new_light = true;
            }
            "LIGHT_INTENSITY" => {
                parser.last_intensity = Some(parse_vec3(&tokens[1..]));
            }
            "LIGHT_DIRECTION" => {
                parser.last_light_direction = Some(parse_vec3(&tokens[1..]));
            }
            "LIGHT_POSITION" => {
                parser.last_light_position = Some(parse_vec3(&tokens[1..]));
            }
            "LIGHT_ATTENUATION" => {
                parser.last_attenuation = Some(parse_vec3(&tokens[1..]));
            }
            _ => {}
        }
    }

    parser.push_light();

    parser.create_scene()
}

fn parse_vec3<'a>(tokens: &[&'a str]) -> Vec3 {
    let r = tokens[0].parse::<f32>().unwrap();
    let g = tokens[1].parse::<f32>().unwrap();
    let b = tokens[2].parse::<f32>().unwrap();

    vec3(r, g, b)
}

fn parse_quaternion<'a>(tokens: &[&'a str]) -> UnitQuaternion<f32> {
    let x = tokens[0].parse::<f32>().unwrap();
    let y = tokens[1].parse::<f32>().unwrap();
    let z = tokens[2].parse::<f32>().unwrap();
    let w = tokens[3].parse::<f32>().unwrap();

    let q = na::Quaternion::<f32>::new(w, x, y, z);
    UnitQuaternion::from_quaternion(q)
}
