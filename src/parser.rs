use glm::{length2, vec3, Vec3};
use na::{Matrix3, Matrix3x1, Matrix3x4, Matrix4, Unit, UnitQuaternion};
use rand::rngs::ThreadRng;
use serde_json;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::path::Path;

use crate::bvh::Bvh;
use crate::camera::Camera;
use crate::objects::*;

pub struct Scene {
    pub camera: Camera,
    pub bvh: Bvh<Primitive>,
    pub lights_bvh: Bvh<Triangle>,
    pub materials: Vec<Material>,
}

pub struct Parameters {
    pub ray_depth: usize,
    pub n_samples: usize,

    pub image_width: usize,
    pub image_height: usize,

    pub background_color: Vec3,
}

#[derive(Default)]
pub struct SceneBuilder {
    camera: Option<Camera>,
    primitives: Vec<Primitive>,
    materials: Vec<Material>,
}

struct SceneJsonData<'a> {
    nodes: &'a [serde_json::Value],
    meshes: &'a [serde_json::Value],
    cameras: &'a [serde_json::Value],

    accessors: &'a [serde_json::Value],
    buffer_views: Vec<BufferView>,
    buffers: Vec<Vec<u8>>,

    image_width: usize,
    image_height: usize,
}

struct BufferView {
    idx: usize,
    offset: usize,
    length: usize,
}

impl SceneBuilder {
    pub fn build_scene(self) -> Scene {
        let lights = self
            .primitives
            .iter()
            .filter_map(|primitive| {
                let emission = self.materials[primitive.material_idx].emission;
                if glm::length2(&emission) == 0.0 {
                    return None;
                } else {
                    Some(primitive.triangle.clone())
                }
            })
            .collect::<Vec<_>>();

        let bvh = Bvh::new(self.primitives);
        println!("====    BVH     ====");
        bvh.print();

        let lights_bvh = Bvh::new(lights);
        println!("==== Lights BVH ====");
        lights_bvh.print();

        Scene {
            camera: self.camera.unwrap(),
            bvh,
            lights_bvh,
            materials: self.materials,
        }
    }
}

pub fn parse_scene(path: &str, width: usize, height: usize) -> Scene {
    let mut builder = SceneBuilder::default();

    let scene_string = std::fs::read_to_string(path).unwrap();
    let scene_json: serde_json::Value = serde_json::from_str(&scene_string).unwrap();

    let scene_data = SceneJsonData {
        buffers: load_buffers(&scene_json, path),
        buffer_views: parse_bufferviews(&scene_json),
        nodes: parse_array(&scene_json, "nodes"),
        meshes: parse_array(&scene_json, "meshes"),
        accessors: parse_array(&scene_json, "accessors"),
        cameras: parse_array(&scene_json, "cameras"),
        image_width: width,
        image_height: height,
    };

    parse_nodes(&mut builder, &scene_data);
    builder.materials = load_materials(&scene_json);

    println!(
        "Loaded {} triangles and {} materials",
        builder.primitives.len(),
        builder.materials.len(),
    );

    builder.build_scene()
}

fn parse_nodes<'a>(scene_builder: &mut SceneBuilder, scene_data: &SceneJsonData<'a>) {
    let nodes = &scene_data.nodes;
    let mut node_parents = vec![None; nodes.len()];

    for (idx, node) in nodes.iter().enumerate() {
        if let serde_json::Value::Array(children) = &node["children"] {
            for child in children {
                let serde_json::Value::Number(child_idx) = child else {
                    panic!()
                };
                let child_idx = child_idx.as_u64().unwrap() as usize;
                node_parents[child_idx] = Some(idx);
            }
        }
    }

    for node_idx in 0..nodes.len() {
        if node_parents[node_idx].is_none() {
            dfs(node_idx, &Matrix4::identity(), scene_builder, scene_data);
        }
    }
}

fn dfs<'a>(
    node_idx: usize,
    parent_transformation: &Matrix4<f32>,
    scene_builder: &mut SceneBuilder,
    scene_data: &SceneJsonData<'a>,
) {
    let node = &scene_data.nodes[node_idx];
    let transformation = parse_transformation(node) * parent_transformation;

    if let Some(camera_idx) = parse_opt_usize(node, "camera") {
        println!("Camera {} found!", camera_idx);
        let aspect = (scene_data.image_width as f32) / (scene_data.image_height as f32);
        let camera = parse_camera(&scene_data.cameras, node, camera_idx, aspect);
        scene_builder.camera = Some(camera);
    }

    if let Some(mesh_idx) = parse_opt_usize(node, "mesh") {
        println!("Mesh {} found!", mesh_idx);
        let mut primitives = load_mesh(
            &scene_data.meshes[mesh_idx],
            &scene_data.accessors,
            &scene_data.buffer_views,
            &scene_data.buffers,
            &transformation,
        );
        scene_builder.primitives.append(&mut primitives);
    }

    if let serde_json::Value::Array(children) = &node["children"] {
        for child in children {
            let serde_json::Value::Number(child_idx) = child else {
                panic!()
            };
            let child_idx = child_idx.as_u64().unwrap() as usize;
            dfs(child_idx, &transformation, scene_builder, scene_data);
        }
    }
}

fn load_mesh(
    mesh_json: &serde_json::Value,
    accessors: &[serde_json::Value],
    buffer_views: &[BufferView],
    buffers: &[Vec<u8>],
    transformation: &Matrix4<f32>,
) -> Vec<Primitive> {
    let serde_json::Value::Array(primitives) = &mesh_json["primitives"] else {
        panic!()
    };

    primitives
        .into_iter()
        .map(|primitive| {
            let attributes = &primitive["attributes"];
            let vertices_acc = parse_usize(attributes, "POSITION");
            let vertices = load_vertices(&accessors[vertices_acc], buffer_views, buffers);
            let vertices = vertices
                .into_iter()
                .map(|v| (transformation * v.push(1.0)).remove_row(3))
                .collect::<Vec<_>>();
            // TODO: normal + tex coord

            let indices_acc = parse_usize(primitive, "indices");
            let indices = load_indices(&accessors[indices_acc], buffer_views, buffers);
            let material_idx = parse_usize(primitive, "material");

            assert_eq!(indices.len() % 3, 0);
            (0..indices.len() / 3).map(move |i| {
                let v1 = vertices[indices[i * 3]];
                let v2 = vertices[indices[i * 3 + 1]];
                let v3 = vertices[indices[i * 3 + 2]];
                Primitive {
                    triangle: Triangle::new(v1, v2, v3),
                    material_idx,
                }
            })
        })
        .flatten()
        .collect()
}

fn load_materials(scene_json: &serde_json::Value) -> Vec<Material> {
    parse_array(scene_json, "materials")
        .into_iter()
        .map(|mat_json| {
            let color_json = &mat_json["pbrMetallicRoughness"]["baseColorFactor"];
            let color_data = if !color_json.is_null() {
                parse_vec(color_json, 4)
            } else {
                vec![1.0; 4]
            };
            let color = vec3(color_data[0], color_data[1], color_data[2]);
            let alpha = color_data[3];
            let ior = 1.5; // TODO: extension?
            let metallic_factor =
                parse_opt_f32(&mat_json["pbrMetallicRoughness"], "metallicFactor").unwrap_or(1.0);
            let emission_json = &mat_json["emissiveFactor"];
            let emission = if !emission_json.is_null() {
                parse_vec(emission_json, 3)
            } else {
                vec![0.0; 3]
            };
            let emission_factor = parse_opt_f32(
                &mat_json["extensions"]["KHR_materials_emissive_strength"],
                "emissiveStrength",
            )
            .unwrap_or(1.0);

            let emission = vec3(emission[0], emission[1], emission[2]) * emission_factor;
            let material_type = if alpha < 1.0 {
                MaterialType::Dielectric { ior }
            } else if metallic_factor > 0.0 {
                MaterialType::Metallic
            } else {
                MaterialType::Diffuse
            };

            Material {
                color,
                emission,
                material_type,
            }
        })
        .collect()
}

fn load_indices(
    accessor: &serde_json::Value,
    buffer_views: &[BufferView],
    buffers: &[Vec<u8>],
) -> Vec<usize> {
    let buff_view_idx = parse_usize(accessor, "bufferView");
    let count = parse_usize(accessor, "count");
    let offset = parse_opt_usize(accessor, "byteOffset").unwrap_or(0);
    let component_type = parse_usize(accessor, "componentType");
    assert_eq!(parse_string(accessor, "type"), "SCALAR");

    let is_16 = if component_type == 5123 {
        true
    } else if component_type == 5125 {
        false
    } else {
        panic!()
    };

    let buff_idx = buffer_views[buff_view_idx].idx;
    let start = offset + buffer_views[buff_view_idx].offset;
    load_byte_usizes(&buffers[buff_idx], start, count, is_16)
}

fn load_vertices(
    accessor: &serde_json::Value,
    buffer_views: &[BufferView],
    buffers: &[Vec<u8>],
) -> Vec<Vec3> {
    let buff_view_idx = parse_usize(accessor, "bufferView");
    let count = parse_usize(accessor, "count");
    let offset = parse_opt_usize(accessor, "byteOffset").unwrap_or(0);
    assert_eq!(parse_usize(accessor, "componentType"), 5126);
    assert_eq!(parse_string(accessor, "type"), "VEC3");

    let buff_idx = buffer_views[buff_view_idx].idx;
    let start = offset + buffer_views[buff_view_idx].offset;
    load_byte_vec3(&buffers[buff_idx], start, count)
}

fn load_byte_vec3(bytes: &[u8], start: usize, count: usize) -> Vec<Vec3> {
    let floats = load_byte_floats(bytes, start, count * 3);
    let mut res = Vec::with_capacity(count);
    for i in 0..count {
        res.push(vec3(floats[3 * i], floats[3 * i + 1], floats[3 * i + 2]));
    }
    res
}

fn load_byte_floats(bytes: &[u8], start: usize, count: usize) -> Vec<f32> {
    (0..count)
        .map(|i| {
            let j = start + i * 4;
            f32::from_le_bytes([bytes[j], bytes[j + 1], bytes[j + 2], bytes[j + 3]])
        })
        .collect()
}

fn load_byte_usizes(bytes: &[u8], start: usize, count: usize, is_16: bool) -> Vec<usize> {
    (0..count)
        .map(|i| {
            if is_16 {
                let j = start + i * 2;
                u16::from_le_bytes([bytes[j], bytes[j + 1]]) as usize
            } else {
                let j = start + i * 4;
                u32::from_le_bytes([bytes[j], bytes[j + 1], bytes[j + 2], bytes[j + 3]]) as usize
            }
        })
        .collect()
}

fn parse_bufferviews(scene_json: &serde_json::Value) -> Vec<BufferView> {
    let serde_json::Value::Array(buffer_views) = &scene_json["bufferViews"] else {
        panic!()
    };

    buffer_views
        .into_iter()
        .map(|bf_view| BufferView {
            idx: parse_usize(&bf_view, "buffer"),
            offset: parse_usize(&bf_view, "byteOffset"),
            length: parse_usize(&bf_view, "byteLength"),
        })
        .collect()
}

fn load_buffers(scene_json: &serde_json::Value, gltf_path: &str) -> Vec<Vec<u8>> {
    let path = Path::new(gltf_path).parent().unwrap();

    let serde_json::Value::Array(buffers) = &scene_json["buffers"] else {
        panic!()
    };

    buffers
        .into_iter()
        .map(|buffer| {
            let serde_json::Value::String(filename) = &buffer["uri"] else {
                panic!()
            };
            let byte_length = parse_usize(&buffer, "byteLength");

            let buffer_path = path.join(filename);
            let buffer_path = buffer_path.to_str().unwrap();
            let file = File::open(buffer_path).unwrap();

            let mut buffer = BufReader::new(file);
            let mut data = vec![0u8; byte_length];
            buffer.read_exact(&mut data).unwrap();
            data
        })
        .collect()
}

fn parse_transformation(node: &serde_json::Value) -> Matrix4<f32> {
    if !node["matrix"].is_null() {
        let data = parse_vec(&node["matrix"], 16);
        return Matrix4::from_vec(data);
    }

    let mut transformation = Matrix4::<f32>::identity();

    let mut mat3 = if !node["scale"].is_null() {
        let scale_vec = parse_vec(&node["scale"], 3);
        Matrix3::from_diagonal(&Matrix3x1::from_vec(scale_vec))
    } else {
        Matrix3::identity()
    };

    if !node["translation"].is_null() {
        let translation = parse_vec(&node["translation"], 3);
        for i in 0..3 {
            transformation[(i, 3)] = translation[i];
        }
    }

    if !node["rotation"].is_null() {
        let r = parse_vec(&node["rotation"], 4);
        let q = na::Quaternion::<f32>::new(r[3], r[0], r[1], r[2]);
        let rotation = UnitQuaternion::from_quaternion(q).to_rotation_matrix();
        mat3 = rotation * mat3;
    }

    for i in 0..3 {
        for j in 0..3 {
            transformation[(i, j)] = mat3[(i, j)];
        }
    }

    transformation
}

fn parse_camera(
    cameras: &[serde_json::Value],
    camera_node: &serde_json::Value,
    camera_idx: usize,
    aspect_x_to_y: f32,
) -> Camera {
    let camera_properties = &cameras[camera_idx]["perspective"];
    let yfov = camera_properties["yfov"].as_f64().unwrap() as f32;
    let yfov = yfov / 2.0;

    let transformation = parse_transformation(camera_node);
    let axis = transformation.fixed_view::<3, 3>(0, 0);
    let position = transformation.fixed_view::<3, 1>(0, 3);

    Camera {
        position: position.into(),
        axis: axis.into(),
        tg_fov_x: (aspect_x_to_y * yfov).tan(),
        tg_fov_y: yfov.tan(),
    }
}

fn parse_vec(json_data: &serde_json::Value, expected_len: usize) -> Vec<f32> {
    let serde_json::Value::Array(data) = json_data else {
        panic!()
    };

    let data = data
        .into_iter()
        .map(|val| {
            let serde_json::Value::Number(val) = val else {
                panic!()
            };
            val.as_f64().unwrap() as f32
        })
        .collect::<Vec<_>>();

    assert_eq!(data.len(), expected_len);
    data
}

fn parse_usize(json_data: &serde_json::Value, key: &str) -> usize {
    let serde_json::Value::Number(val) = &json_data[key] else {
        panic!()
    };
    val.as_u64().unwrap() as usize
}

fn parse_opt_usize(json_data: &serde_json::Value, key: &str) -> Option<usize> {
    let serde_json::Value::Number(val) = &json_data[key] else {
        return None;
    };
    Some(val.as_u64().unwrap() as usize)
}

fn parse_opt_f32(json_data: &serde_json::Value, key: &str) -> Option<f32> {
    let serde_json::Value::Number(val) = &json_data[key] else {
        return None;
    };
    Some(val.as_f64().unwrap() as f32)
}

fn parse_string(json_data: &serde_json::Value, key: &str) -> String {
    let serde_json::Value::String(string) = &json_data[key] else {
        panic!()
    };
    string.into()
}

fn parse_array<'a>(json_data: &'a serde_json::Value, key: &str) -> &'a [serde_json::Value] {
    let serde_json::Value::Array(array) = &json_data[key] else {
        panic!();
    };
    array
}
