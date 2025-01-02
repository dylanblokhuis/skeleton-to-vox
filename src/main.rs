use bevy::{math::bounding::{Aabb3d, BoundingVolume}, prelude::*, window::WindowResolution};
use bevy_flycam::prelude::*;

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
        speed: 100.0, // default: 12.0
    })
    .add_plugins(NoCameraPlayerPlugin)
    .add_systems(Startup, yo).run();
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

    let mut depth: usize = 0;
    // println!("Root node: {:?}", root.name());
    something(root_armature, depth, &mut voxel_aabbs, None, Transform {
        translation: Vec3::ZERO,
        rotation: Quat::IDENTITY,
        scale: Vec3::ONE,
    });

    for voxel in voxel_aabbs.iter() {
        println!("Voxel: {:?}", voxel);
        // shape::Cube
        commands.spawn((
            Mesh3d(meshes.add(Mesh::from(Cuboid::from_size((voxel.aabb.half_size() * 2.0).into())))),
            Transform::from_translation(voxel.aabb.center().into()),
            MeshMaterial3d(materials.add(Color::srgb_u8(rand::random(), rand::random(), rand::random()))),
        ));
    }

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
        FlyCam
    ));
}

fn something(
    node: gltf::Node,
    depth: usize,
    voxel_aabbs: &mut Vec<VoxelObject>,
    parent_global_transform: Option<Transform>,
    parent_position: Transform,
) {
    let local_transform = transform_from_gltf(node.transform());
    let global_transform =
        parent_global_transform.map_or(local_transform, |parent| parent * local_transform);

    // Current bone's end position (global position)
    let end_position = global_transform.translation;

    // If there's a parent, create a VoxelObject spanning between the parent and current node
    if let Some(parent_transform) = parent_global_transform {
        // let min = parent_transform.translation;
        // let max = end_position;

        let center = (end_position + parent_transform.translation) / 2.0;
        let half_extents = (end_position - parent_transform.translation).abs() / 2.0;

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
            Transform::from_translation(end_position),
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
