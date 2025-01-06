use std::{fs::File, io::BufWriter};

use bevy::{
    math::bounding::{Aabb3d, BoundingVolume},
    prelude::*,
    window::WindowResolution,
};
use bevy_flycam::prelude::*;
use dot_vox::DotVoxData;
use gltf::{buffer, image, Document};
use slab::Slab;

#[derive(Debug)]
struct VoxelObject {
    name: String,
    aabb: Aabb3d,
    children: Vec<usize>,
}

// Thickness of the bone voxel, whatever axis is not the longest will be this thick
const BONE_VOXEL_THICKNESS: f32 = 10.0;

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
        .add_systems(Startup, setup_camera)
        .add_systems(FixedUpdate, convert)
        .run();
}

// struct VoxAnimationTree {
//     vox_data: Vec<u8>,

// }

struct GltfScene((Document, Vec<buffer::Data>, Vec<image::Data>));

fn convert(
    // mut commands: Commands,
    // mut meshes: ResMut<Assets<Mesh>>,
    // mut materials: ResMut<Assets<StandardMaterial>>,
    mut gizmos: Gizmos,
    mut scene: Local<Option<GltfScene>>,
) {
    if scene.is_none() {
        let (document, data, _images) = gltf::import("./input/Fox.gltf").unwrap();
        *scene = Some(GltfScene((document, data, _images)));
    }

    let GltfScene((document, _data, _images)) = scene.as_ref().unwrap();
    // let (document, data, _images) = gltf::import("./input/Fox.gltf").unwrap();

    let skin = document.skins().next().unwrap();
    // let joint_indices = skin.joints().map(|joint| joint.index()).collect::<Vec<_>>();

    fn do_skin(node: gltf::Node, skin: &gltf::Skin, depth: usize, gizmos: &mut Gizmos, parent_transform: Transform) {
        let joint = skin.joints().find(|joint| joint.index() == node.index());
        if joint.is_none() {
            println!("found non joint as node, we stop recursion");
            return;
        }
        let transform = transform_from_gltf(node.transform());
        let transform = parent_transform * transform;
        let depth_as_string = std::iter::repeat("  ").take(depth).collect::<String>();
        println!(
            "{}Joint: {:?} - {:?}",
            depth_as_string,
            node.name().unwrap(),
            transform
        );
        gizmos.cuboid(
            transform,
            Color::linear_rgb(1.0, 0.0, 0.0),
        );

        for child in node.children() {
            do_skin(child, skin, depth + 1, gizmos, transform);
        }
    }

    do_skin(skin.joints().next().unwrap(), &skin, 0, &mut gizmos, Transform::default());

    // let joint_ids
    // skin.joints().for_each(|joint| {

    //     println!("Joint: {:?} {:?} {:?}", joint.name(), transform_from_gltf(joint.transform()), joint.children());
    // });

    // let default_scene = document.default_scene().unwrap();

    // fn create_stuff(
    //     node: gltf::Node,
    //     commands: &mut Commands,
    //     meshes: &mut ResMut<Assets<Mesh>>,
    //     materials: &mut ResMut<Assets<StandardMaterial>>,
    //     parent: Option<Entity>
    // ) -> Entity {
    //     let entity = commands.spawn((
    //         transform_from_gltf(node.transform()),
    //         Mesh3d(meshes.add(Mesh::from(Cuboid::from_size(Vec3::splat(1.0))))),
    //         MeshMaterial3d(materials.add(Color::srgb_u8(
    //             rand::random(),
    //             rand::random(),
    //             rand::random(),
    //         ))),
    //     )).id();
    //     if let Some(parent) = parent {
    //         commands.entity(parent).add_child(entity);
    //     }
    //     println!("Node: {:?}", node.name());

    //     for child in node.children() {
    //         let child_entity = create_stuff(child, commands, meshes, materials, parent);
    //         commands.entity(entity).add_child(child_entity);
    //     }

    //     entity
    // }

    // create_stuff(default_scene.nodes().next().unwrap(), &mut commands, &mut meshes, &mut materials, None);

    // for animation in document.animations() {
    //     for channel in animation.channels() {
    //         let reader = channel.reader(|buffer| Some(&data[buffer.index()]));

    //         let target = channel.target();
    //         println!("Target: {:?}", target.node().name());

    //     }
    // }

    // for node in default_scene.nodes() {
    //     let parent = commands.spawn((
    //         transform_from_gltf(node.transform()),
    //         Mesh3d(meshes.add(Mesh::from(Cuboid::from_size(Vec3::splat(1.0))))),
    //     ));
    //     println!("Node: {:?}", node.name());
    // }

    // for node in document
    // .nodes() {
    //     println!("Node: {:?}", node.name());
    // }

    // let armature_name = "b_Hip_01";
    // let root_armature = document
    //     .nodes()
    //     .find(|node| node.name().unwrap() == armature_name)
    //     .unwrap();

    // let mut voxel_aabbs: Slab<VoxelObject> = Slab::new();

    // create_voxel_aabbs_from_skeleton(root_armature, 0, &mut voxel_aabbs, None);

    // for (_, voxel) in voxel_aabbs.iter() {
    //     println!("Voxel: {:?}", voxel);
    //     // shape::Cube
    //     commands.spawn((
    //         Mesh3d(meshes.add(Mesh::from(Cuboid::from_size(
    //             (voxel.aabb.half_size() * 2.0).into(),
    //         )))),
    //         Transform::from_translation(voxel.aabb.center().into()),
    //         MeshMaterial3d(materials.add(Color::srgb_u8(
    //             rand::random(),
    //             rand::random(),
    //             rand::random(),
    //         ))),
    //     ));
    // }

    // let mut dot_vox_data = DotVoxData {
    //     version: 150,
    //     layers: vec![dot_vox::Layer {
    //         attributes: Default::default(),
    //     }],
    //     models: vec![],
    //     palette: vec![
    //         dot_vox::Color {
    //             r: 0,
    //             g: 0,
    //             b: 0,
    //             a: 255,
    //         };
    //         256
    //     ],
    //     materials: vec![],
    //     scenes: vec![
    //         dot_vox::SceneNode::Transform {
    //             attributes: Default::default(),
    //             child: 1,
    //             layer_id: 4294967295,
    //             frames: vec![dot_vox::Frame {
    //                 attributes: Default::default(),
    //             }],
    //         },
    //         dot_vox::SceneNode::Group {
    //             attributes: Default::default(),
    //             children: vec![],
    //         },
    //     ],
    // };

    // for material_idx in 0..256 {
    //     let mut material = dot_vox::Material {
    //         id: material_idx as u32,
    //         properties: Default::default(),
    //     };

    //     let properties = [
    //         ("_rough", "0.1"),
    //         ("_ior", "0.3"),
    //         ("_spec", "0.5"),
    //         ("_weight", "1"),
    //         ("_type", "_diffuse"),
    //     ];

    //     for (k, v) in properties.iter() {
    //         material.properties.insert(k.to_string(), v.to_string());
    //     }

    //     dot_vox_data.materials.push(material);
    // }

    // let layer_idx = 0;
    // let mut transform_nodes_indices = vec![];

    // for (_, object) in voxel_aabbs.iter() {
    //     let u_size = (object.aabb.half_size() * 2.0).as_uvec3();
    //     let model: dot_vox::Model = dot_vox::Model {
    //         // magicavoxel uses xzy
    //         size: dot_vox::Size {
    //             x: u_size.x as u32,
    //             y: u_size.z as u32,
    //             z: u_size.y as u32,
    //         },
    //         voxels: vec![],
    //     };
    //     dot_vox_data.models.push(model);

    //     let mut frame = dot_vox::Frame {
    //         attributes: Default::default(),
    //     };
    //     frame.attributes.insert(
    //         "_t".to_string(),
    //         format!(
    //             "{} {} {}",
    //             object.aabb.center().x as i32,
    //             object.aabb.center().z as i32,
    //             object.aabb.center().y as i32
    //         ),
    //     );

    //     let mut transform_dict = dot_vox::Dict::new();
    //     transform_dict.insert("_name".to_string(), object.name.clone());
    //     dot_vox_data.scenes.push(dot_vox::SceneNode::Transform {
    //         attributes: transform_dict,
    //         frames: vec![frame],
    //         // shape id
    //         child: dot_vox_data.scenes.len() as u32 + 1,
    //         layer_id: layer_idx,
    //     });

    //     transform_nodes_indices.push(dot_vox_data.scenes.len() as u32 - 1);

    //     dot_vox_data.scenes.push(dot_vox::SceneNode::Shape {
    //         attributes: Default::default(),
    //         models: vec![dot_vox::ShapeModel {
    //             model_id: (dot_vox_data.models.len() - 1) as u32,
    //             attributes: Default::default(),
    //         }],
    //     });
    // }
    // dot_vox_data.scenes[1] = dot_vox::SceneNode::Group {
    //     attributes: Default::default(),
    //     children: transform_nodes_indices,
    // };

    // println!("dot_vox_data: {:?}", dot_vox_data.scenes);
    // let mut vox_file = BufWriter::new(File::create("output.vox").unwrap());
    // dot_vox_data.write_vox(&mut vox_file).unwrap();

    // commands.spawn((
    //     DirectionalLight {
    //         // shadows_enabled: true,
    //         ..Default::default()
    //     },
    //     Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4)),
    // ));

    // commands.spawn((
    //     Camera3d::default(),
    //     Transform::from_xyz(-2.5, 10.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
    //     FlyCam,
    // ));
}

fn setup_camera(
   mut commands: Commands
) {
    commands.spawn((
        DirectionalLight {
            // shadows_enabled: true,
            ..Default::default()
        },
        Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4)),
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-2.5, 100.0, 100.0).looking_at(Vec3::ZERO, Vec3::Y),
        FlyCam,
    ));
}

fn create_voxel_aabbs_from_skeleton(
    node: gltf::Node,
    depth: usize,
    dot_vox_data: &mut DotVoxData,
    parent_global_transform: Transform,
) -> u32 {
    let local_transform = transform_from_gltf(node.transform());
    let global_transform = parent_global_transform * local_transform;

    // Current bone's end position (global position)
    let end_position = global_transform.translation;

    // If there's a parent, create a VoxelObject spanning between the parent and current node
    let center = Vec3::new(
        (end_position.x + parent_global_transform.translation.x) / 2.0,
        (end_position.z + parent_global_transform.translation.z) / 2.0,
        (end_position.y + parent_global_transform.translation.y) / 2.0,
    );

    let half_extents = Vec3::new(
        (end_position.x - parent_global_transform.translation.x).abs() / 2.0,
        (end_position.z - parent_global_transform.translation.z).abs() / 2.0,
        (end_position.y - parent_global_transform.translation.y).abs() / 2.0,
    );

    // we have to make sure the AABB has atleast a size of 1
    let half_extents = half_extents.max(Vec3::splat(BONE_VOXEL_THICKNESS / 2.0));

    let u_size = (half_extents * 2.0).as_uvec3();
    let model: dot_vox::Model = dot_vox::Model {
        // magicavoxel uses xzy
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
            center.x as i32, center.z as i32, center.y as i32
        ),
    );

    let mut transform_dict = dot_vox::Dict::new();
    transform_dict.insert("_name".to_string(), node.name().unwrap().to_string());
    dot_vox_data.scenes.push(dot_vox::SceneNode::Transform {
        attributes: transform_dict,
        frames: vec![frame],
        // shape id
        child: dot_vox_data.scenes.len() as u32 + 1,
        layer_id: 0,
    });

    let idx = dot_vox_data.scenes.len() as u32 - 1;

    dot_vox_data.scenes.push(dot_vox::SceneNode::Shape {
        attributes: Default::default(),
        models: vec![dot_vox::ShapeModel {
            model_id: (dot_vox_data.models.len() - 1) as u32,
            attributes: Default::default(),
        }],
    });

    // Some(voxel_aabbs.insert(VoxelObject {
    //     name: node.name().unwrap().to_string(),
    //     aabb: Aabb3d::new(center, half_extents),
    // }))

    let depth_as_string = std::iter::repeat("  ").take(depth).collect::<String>();
    println!(
        "{}Node: {:?} {:?}",
        depth_as_string,
        node.name().unwrap(),
        global_transform
    );

    let mut children = vec![];
    for child in node.children() {
        children.push(create_voxel_aabbs_from_skeleton(
            child,
            depth + 1,
            dot_vox_data,
            global_transform,
        ));
    }

    dot_vox_data.scenes.push(dot_vox::SceneNode::Group {
        attributes: Default::default(),
        children,
    });

    idx
    // scene group
}

pub fn transform_from_gltf(transform: gltf::scene::Transform) -> Transform {
    let (translation, rotation, scale) = transform.decomposed();

    Transform {
        translation: Vec3::from(translation),
        rotation: Quat::from_array(rotation),
        scale: Vec3::from(scale),
    }
}
