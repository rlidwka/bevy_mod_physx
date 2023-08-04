mod common;

use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::render_resource::PrimitiveTopology;
use bevy_mod_physx::assets::GeometryInner;
use bevy_mod_physx::physx_extras::ConvexMeshExtras;
use bevy_mod_physx::prelude::{self as bpx, *};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PhysicsPlugins)
        .add_plugins(common::DemoUtils) // optional
        .add_systems(Startup, (
            spawn_scene,
            spawn_camera_and_light,
        ))
        .run();
}

// generates bevy mesh from physx convex mesh
fn create_bevy_mesh_from_geometry(geometry: &Geometry) -> Mesh {
    let mut positions = vec![];
    let mut indices = vec![];
    let mut normals = vec![];

    match &geometry.obj {
        GeometryInner::ConvexMesh { mesh, scale, .. } => {
            let mesh = mesh.lock().unwrap();

            let mut vertices = vec![];
            // arbitrary axis rotation not implemented
            assert_eq!(scale.rotation.to_bevy(), Quat::IDENTITY);
            for vertex in mesh.get_vertices() {
                vertices.push(vertex.to_bevy() * scale.scale.to_bevy());
            }

            let index_buffer = mesh.get_index_buffer();
            for idx in 0..mesh.get_nb_polygons() {
                let polygon = mesh.get_polygon_data(idx).unwrap();
                for i in polygon.index_base+1..polygon.index_base+polygon.nb_verts-1 {
                    let a = vertices[index_buffer[polygon.index_base as usize] as usize];
                    let b = vertices[index_buffer[i as usize] as usize];
                    let c = vertices[index_buffer[i as usize + 1] as usize];

                    indices.push(indices.len() as u32);
                    indices.push(indices.len() as u32);
                    indices.push(indices.len() as u32);

                    positions.push(a);
                    positions.push(b);
                    positions.push(c);

                    let normal = (b - a).cross(c - a).normalize();
                    normals.push(normal);
                    normals.push(normal);
                    normals.push(normal);
                }
            }
        },

        // see https://github.com/rlidwka/bevy_mod_physx/blob/2cfc2e55db7509fa881c3687800472fa73310361/src/render.rs
        // for inspiration how to implement other shapes
        _ => unimplemented!()
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.set_indices(Some(Indices::U32(indices)));
    mesh
}

pub fn spawn_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut physics: ResMut<Physics>,
    mut px_geometries: ResMut<Assets<bpx::Geometry>>,
) {
    // plane
    commands.spawn((
        RigidBody::Static,
        bpx::Shape {
            geometry: px_geometries.add(bpx::Geometry::halfspace(Vec3::Y)),
            ..default()
        },
        PbrBundle {
            mesh: meshes.add(shape::Plane::from_size(1000.0).into()),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            ..default()
        }
    ));

    // cylinder (it's a helper for convex shape, physx doesn't have native cylinders)
    let geometry = bpx::Geometry::cylinder(&mut physics, 0.5, 0.5, 10).unwrap();
    let mesh = create_bevy_mesh_from_geometry(&geometry);
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(mesh),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(-2.0, 7.0, 0.0).with_rotation(Quat::from_rotation_z(-1.)),
            ..default()
        },
        RigidBody::Dynamic,
        bpx::Shape {
            geometry: px_geometries.add(geometry),
            ..default()
        },
    ));

    // arbitrary convex shape
    let vertices = vec![
        Vec3::new(0., 1., 0.),
        Vec3::new(0., -1., 0.),
        Vec3::new(1., 0., 0.),
        Vec3::new(-1., 0., 0.),
        Vec3::new(0., 0., 1.),
        Vec3::new(0., 0., -1.),
    ];

    let geometry = bpx::Geometry::convex_mesh(&mut physics, &vertices).unwrap()
        .with_scale(Vec3::splat(0.8), Quat::IDENTITY);
    let mesh = create_bevy_mesh_from_geometry(&geometry);
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(mesh),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(2.0, 7.0, 0.0).with_rotation(Quat::from_rotation_z(-1.)),
            ..default()
        },
        RigidBody::Dynamic,
        bpx::Shape {
            geometry: px_geometries.add(geometry),
            ..default()
        },
    ));
}

fn spawn_camera_and_light(mut commands: Commands) {
    commands
        .spawn(SpatialBundle::from_transform(Transform::from_xyz(0., 2.5, 0.)))
        .with_children(|builder| {
            builder.spawn(Camera3dBundle {
                transform: Transform::from_xyz(0.0, 2.5, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
                ..default()
            });
        })
        .insert(Name::new("Camera"));

    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_4)),
        ..default()
    }).insert(Name::new("Light"));
}
