mod common;

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_mod_physx::prelude::{self as bpx, *};

#[derive(Resource)]
struct DemoMaterials {
    normal: Handle<StandardMaterial>,
    highlighted: Handle<StandardMaterial>,
}

#[derive(Component)]
struct Highlightable;

#[derive(Component)]
#[component(storage = "SparseSet")]
struct Highlighted;

fn main() {
    // ported from ray_casting3 example from bevy_rapier3d
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PhysicsPlugins.set(
            PhysicsCore::new().with_pvd()
        ))
        .add_plugins(common::DemoUtils) // optional
        .add_systems(Startup, (
            init_materials,
            ApplyDeferred,
            (
                spawn_plane,
                spawn_cubes,
                spawn_camera_and_light,
            ),
        ).chain())
        .add_systems(Update, (
            hover_reset,
            hover_highlight,
        ).chain())
        .run();
}

fn init_materials(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(DemoMaterials {
        normal: materials.add(Color::srgb(0.8, 0.7, 0.6)),
        highlighted: materials.add(Color::srgb(0.3, 0.4, 0.9)),
    });
}

fn spawn_plane(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut physics: ResMut<Physics>,
    mut px_geometries: ResMut<Assets<bpx::Geometry>>,
    mut px_materials: ResMut<Assets<bpx::Material>>,
) {
    let primitive = Plane3d::default();
    let mesh = meshes.add(primitive.mesh().size(500., 500.));
    let material = materials.add(Color::srgb(0.3, 0.5, 0.3));
    let px_geometry = px_geometries.add(primitive);
    let px_material = px_materials.add(bpx::Material::new(&mut physics, 0.5, 0.5, 0.6));

    commands.spawn_empty()
        .insert((
            Mesh3d::from(mesh.clone()),
            MeshMaterial3d::from(material.clone()),
        ))
        .insert(bpx::RigidBody::Static)
        .insert(bpx::Shape {
            geometry: px_geometry,
            material: px_material,
            ..default()
        })
        .insert(Name::new("Plane"));
}

fn spawn_cubes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<DemoMaterials>,
    mut px_geometries: ResMut<Assets<bpx::Geometry>>,
) {
    let num = 8;
    let rad = 1.0;
    let primitive = Cuboid::from_size(Vec3::splat(rad * 2.));
    let px_geometry = px_geometries.add(primitive);
    let mesh = meshes.add(primitive);
    let material = materials.normal.clone();

    let shift = rad * 2.0 + rad;
    let centerx = shift * (num / 2) as f32;
    let centery = shift / 2.0;
    let centerz = shift * (num / 2) as f32;

    let mut offset = -(num as f32) * (rad * 2.0 + rad) * 0.5;

    for j in 0usize..20 {
        for i in 0..num {
            for k in 0usize..num {
                let x = i as f32 * shift - centerx + offset;
                let y = j as f32 * shift + centery + 3.0;
                let z = k as f32 * shift - centerz + offset;

                commands.spawn((
                    Mesh3d::from(mesh.clone()),
                    MeshMaterial3d::from(material.clone()),
                    Transform::from_xyz(x, y, z),
                    RigidBody::Dynamic,
                    bpx::Shape {
                        geometry: px_geometry.clone(),
                        ..default()
                    },
                    Highlightable,
                ));
            }
        }

        offset -= 0.05 * rad * (num as f32 - 1.0);
    }
}

fn hover_reset(
    mut commands: Commands,
    materials: Res<DemoMaterials>,
    highlighted: Query<Entity, With<Highlighted>>,
) {
    for entity in highlighted.iter() {
        commands.entity(entity)
            .insert(MeshMaterial3d::from(materials.normal.clone()))
            .remove::<Highlighted>();
    }
}

fn hover_highlight(
    mut commands: Commands,
    materials: Res<DemoMaterials>,
    scene: Res<bpx::Scene>,
    window: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    highlighable: Query<(), With<Highlightable>>,
) {
    let Ok(window) = window.single() else { return; };
    let Some(cursor_position) = window.cursor_position() else { return; };

    for (camera, camera_transform) in &cameras {
        let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position).ok() else { continue; };

        if let Some(hit) = scene.raycast(ray, f32::MAX, &default()) {
            if highlighable.get(hit.actor).is_ok() {
                commands.entity(hit.actor)
                    .insert(MeshMaterial3d::from(materials.highlighted.clone()))
                    .insert(Highlighted);
            }
        }
    }
}

fn spawn_camera_and_light(mut commands: Commands) {
    commands
        .spawn((
            Name::new("Camera"),
            Transform::from_xyz(-29., 8.5, -17.2),
            Visibility::default(),
        ))
        .with_children(|builder| {
            builder.spawn((
                Camera3d::default(),
                Transform::from_xyz(-61., 47., 82.).looking_at(Vec3::ZERO, Vec3::Y),
            ));
        });

    commands.spawn((
        Name::new("Light"),
        DirectionalLight::default(),
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -1.2, -0.2, 0.)),
    ));
}
