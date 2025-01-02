use std::{fs::File, io::BufWriter};

use bevy::{
    math::bounding::{Aabb3d, BoundingVolume},
    prelude::*,
    window::WindowResolution,
};
use bevy_flycam::prelude::*;
use dot_vox::DotVoxData;

#[derive(Debug)]
struct VoxelObject {
    name: String,
    aabb: Aabb3d,
}

fn main() {
    App::new()
        // window size 640x480
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: WindowResolution::new(640.0, 480.0),
                ..Default::default()
            }),
            ..Default::default()
        }))
        .insert_resource(MovementSettings {
            sensitivity: 0.00015, // default: 0.00012
            speed: 100.0,         // default: 12.0
        })
        .add_plugins(NoCameraPlayerPlugin)
        .add_systems(Startup, yo)
        .run();
}

fn yo(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let (document, data, images) = gltf::import("./input/Fox.gltf").unwrap();

    let armature_name = "b_Hip_01";
    let root_armature = document
        .nodes()
        .find(|node| node.name().unwrap() == armature_name)
        .unwrap();

    let mut voxel_aabbs: Vec<VoxelObject> = Vec::new();

    something(root_armature, 0, &mut voxel_aabbs, None);

    for voxel in voxel_aabbs.iter() {
        println!("Voxel: {:?}", voxel);
        // shape::Cube
        commands.spawn((
            Mesh3d(meshes.add(Mesh::from(Cuboid::from_size(
                (voxel.aabb.half_size() * 2.0).into(),
            )))),
            Transform::from_translation(voxel.aabb.center().into()),
            MeshMaterial3d(materials.add(Color::srgb_u8(
                rand::random(),
                rand::random(),
                rand::random(),
            ))),
        ));
    }

    let mut henk = dot_vox::load("input/3x3x3.vox").unwrap();

    println!("henk: {:?}", henk);

    let mut dot_vox_data = DotVoxData {
        version: 150,
        layers: vec![dot_vox::Layer {
            attributes: Default::default(),
        }],
        models: vec![],
        palette: vec![dot_vox::Color {
            r: rand::random(),
            g: rand::random(),
            b: rand::random(),
            a: 255,
        }; 256],
        materials: vec![],
        scenes: vec![
            dot_vox::SceneNode::Transform {
                attributes: Default::default(),
                child: 1,
                layer_id: 4294967295,
                frames: vec![dot_vox::Frame {
                    attributes: Default::default(),
                }],
            },
            dot_vox::SceneNode::Group {
                attributes: Default::default(),
                children: vec![],
            },
        ],
    };

    for material_idx in 0..256 {
        let mut material = dot_vox::Material {
            id: material_idx as u32,
            properties: Default::default(),
        };

        let properties = [
            ("_rough", "0.1"),
            ("_ior", "0.3"),
            ("_spec", "0.5"),
            ("_weight", "1"),
            ("_type", "_diffuse"),
        ];

        for (k, v) in properties.iter() {
            material.properties.insert(k.to_string(), v.to_string());
        }

        dot_vox_data.materials.push(material);
    }

    let layer_idx = 0;
    let mut transform_nodes_indices = vec![];

    for object in voxel_aabbs.iter() {
        let u_size = (object.aabb.half_size() * 2.0).as_uvec3();
        let model: dot_vox::Model = dot_vox::Model {
            size: dot_vox::Size {
                x: u_size.x as u32,
                y: u_size.z as u32,
                z: u_size.y as u32,
            },
            voxels: vec![],
        };
        dot_vox_data.models.push(model);

        let mut frame = dot_vox::Frame {
            attributes: Default::default(),
        };
        frame.attributes.insert(
            "_t".to_string(),
            format!(
                "{} {} {}",
                object.aabb.center().x as i32,
                object.aabb.center().z as i32,
                object.aabb.center().y as i32
            ),
        );

        println!("frame: {:?}", frame);
        
        let mut transform_dict =  dot_vox::Dict::new();
        transform_dict.insert("_name".to_string(), object.name.clone());
        dot_vox_data.scenes.push(dot_vox::SceneNode::Transform {
            attributes: transform_dict,
            frames: vec![frame],
            // shape id
            child: dot_vox_data.scenes.len() as u32 + 1,
            layer_id: layer_idx,
        });

        transform_nodes_indices.push(dot_vox_data.scenes.len() as u32 - 1);

        dot_vox_data.scenes.push(
            dot_vox::SceneNode::Shape {
                attributes: Default::default(),
                models: vec![dot_vox::ShapeModel {
                    model_id: (dot_vox_data.models.len() - 1) as u32,
                    attributes: Default::default(),
                }]
            }
        );
    }
    dot_vox_data.scenes[1] = dot_vox::SceneNode::Group {
        attributes: Default::default(),
        children: transform_nodes_indices,
    };


    println!("dot_vox_data: {:?}", dot_vox_data.scenes);
    let mut vox_file = BufWriter::new(File::create("output.vox").unwrap());
    dot_vox_data.write_vox(&mut vox_file).unwrap();

    commands.spawn((
        DirectionalLight {
            // shadows_enabled: true,
            ..Default::default()
        },
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4)),
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 200.0, 100.0).looking_at(Vec3::ZERO, Vec3::Y),
        FlyCam,
    ));
}

fn something(
    node: gltf::Node,
    depth: usize,
    voxel_aabbs: &mut Vec<VoxelObject>,
    parent_global_transform: Option<Transform>,
) {
    let local_transform = transform_from_gltf(node.transform());
    let global_transform =
        parent_global_transform.map_or(local_transform, |parent| parent * local_transform);

    // Current bone's end position (global position)
    let end_position = global_transform.translation;

    // If there's a parent, create a VoxelObject spanning between the parent and current node
    if let Some(parent_transform) = parent_global_transform {
        let center = Vec3::new(
            (end_position.x + parent_transform.translation.x) / 2.0,
            (end_position.z + parent_transform.translation.z) / 2.0,
            (end_position.y + parent_transform.translation.y) / 2.0,
        );
        
        let half_extents = Vec3::new(
            (end_position.x - parent_transform.translation.x).abs() / 2.0,
            (end_position.z - parent_transform.translation.z).abs() / 2.0,
            (end_position.y - parent_transform.translation.y).abs() / 2.0,
        );
        

        // we have to make sure the AABB has atleast a size of 1
        let half_extents = half_extents.max(Vec3::ONE);

        voxel_aabbs.push(VoxelObject {
            name: node.name().unwrap().to_string(),
            aabb: Aabb3d::new(center, half_extents),
        });
    }

    let depth_as_string = std::iter::repeat("  ").take(depth).collect::<String>();
    println!(
        "{}Node: {:?} {:?}",
        depth_as_string,
        node.name().unwrap(),
        global_transform
    );

    for child in node.children() {
        something(
            child,
            depth + 1,
            voxel_aabbs,
            Some(global_transform),
        );
    }
}

pub fn transform_from_gltf(transform: gltf::scene::Transform) -> Transform {
    let (translation, rotation, scale) = transform.decomposed();

    Transform {
        translation: Vec3::from(translation),
        rotation: Quat::from_array(rotation),
        scale: Vec3::from(scale),
    }
}
