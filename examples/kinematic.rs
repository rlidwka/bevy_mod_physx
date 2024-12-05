mod common;

use std::f32::consts::FRAC_PI_2;
use std::ffi::c_void;

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_mod_physx::prelude::{self as bpx, *};
use bevy_mod_physx::utils::get_actor_entity_from_ptr;
use bevy_mod_physx::utils::raycast::SceneQueryFilter;
use physx_sys::{PxFilterData, PxQueryHitType, PxRigidActor, PxShape};

#[derive(Component)]
struct Surface;

#[derive(Component)]
struct PlayerControlled;

const BALL_SIZE: f32 = 0.5;
const CUE_SIZE: f32 = 0.5;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PhysicsPlugins.set(
            PhysicsCore::new().with_pvd()
        ))
        .add_plugins(common::DemoUtils) // optional
        .add_systems(Startup, (
            spawn_table,
            spawn_pyramid,
            spawn_kinematic,
            spawn_camera_and_light,
        ))
        .add_systems(Update, move_kinematic)
        .run();
}

fn spawn_table(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut physics: ResMut<Physics>,
    mut px_geometries: ResMut<Assets<bpx::Geometry>>,
    mut px_materials: ResMut<Assets<bpx::Material>>,
) {
    const SIZE: f32 = 40.;
    const THICKNESS: f32 = 0.5;
    const SIDE_HEIGHT: f32 = 1.;

    let primitive = Cuboid::new(SIZE, THICKNESS, SIZE);
    let mesh = meshes.add(primitive);
    let material = materials.add(Color::srgb(0.3, 0.5, 0.3));
    let px_geometry = px_geometries.add(primitive);
    let px_material = px_materials.add(bpx::Material::new(&mut physics, 0.5, 0.5, 0.6));

    commands.spawn_empty()
        .insert((
            Mesh3d::from(mesh.clone()),
            MeshMaterial3d::from(material.clone()),
            Transform::from_xyz(0., -THICKNESS / 2., 0.),
        ))
        .insert(bpx::RigidBody::Static)
        .insert(bpx::Shape {
            geometry: px_geometry,
            material: px_material,
            ..default()
        })
        .insert(Surface)
        .insert(Name::new("TableSurface"));

    let primitive = Cuboid::new(SIZE + THICKNESS, SIDE_HEIGHT, THICKNESS);
    let mesh = meshes.add(primitive);
    let material = materials.add(Color::srgb(0.3, 0.5, 0.3));
    let px_geometry = px_geometries.add(primitive);
    let px_material = px_materials.add(bpx::Material::new(&mut physics, 0.5, 0.5, 0.6));

    for side in 0..4 {
        commands.spawn_empty()
            .insert((
                Mesh3d::from(mesh.clone()),
                MeshMaterial3d::from(material.clone()),
                Transform {
                    translation: Quat::from_rotation_y((side + 1) as f32 * FRAC_PI_2) * Vec3::new(SIZE / 2., THICKNESS / 2., 0.),
                    rotation: Quat::from_rotation_y(side as f32 * FRAC_PI_2),
                    scale: Vec3::ONE,
                },
            ))
            .insert(bpx::RigidBody::Static)
            .insert(bpx::Shape {
                geometry: px_geometry.clone(),
                material: px_material.clone(),
                ..default()
            })
            .insert(Name::new("TableSide"));
    }
}

fn spawn_pyramid(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut physics: ResMut<Physics>,
    mut px_geometries: ResMut<Assets<bpx::Geometry>>,
    mut px_materials: ResMut<Assets<bpx::Material>>,
) {
    let primitive = Sphere::new(BALL_SIZE);
    let mesh = meshes.add(primitive);
    let material = materials.add(Color::srgb(0.8, 0.7, 0.6));

    let px_geometry = px_geometries.add(primitive);
    let px_material = px_materials.add(bpx::Material::new(&mut physics, 0., 0., 1.));

    for dx in 0..8 {
        for dy in 0..=dx {
            let x = dx as f32 * BALL_SIZE * 2. + 7.5;
            let y = (dy as f32 - dx as f32 / 2.) * BALL_SIZE * 2.;

            commands.spawn_empty()
                .insert((
                    Mesh3d::from(mesh.clone()),
                    MeshMaterial3d::from(material.clone()),
                    Transform::from_xyz(x, BALL_SIZE / 2., y),
                ))
                .insert(bpx::RigidBody::Dynamic)
                .insert(bpx::Shape {
                    material: px_material.clone(),
                    geometry: px_geometry.clone(),
                    ..default()
                })
                // add max velocity to prevent things from flying
                // off the screen with too fast mouse movement
                .insert(MaxVelocity::linear(40.))
                .insert(Name::new("Ball"));
        }
    }
}

fn spawn_kinematic(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut physics: ResMut<Physics>,
    mut px_geometries: ResMut<Assets<bpx::Geometry>>,
    mut px_materials: ResMut<Assets<bpx::Material>>,
) {
    let primitive = Cuboid::new(CUE_SIZE, CUE_SIZE, CUE_SIZE);
    let mesh = meshes.add(primitive);
    let material = materials.add(Color::srgb(0.8, 0.7, 0.6));

    let px_geometry = px_geometries.add(primitive);
    let px_material = px_materials.add(bpx::Material::new(&mut physics, 0., 0., 1.));
    let transform = Transform::from_xyz(0., BALL_SIZE, 0.);

    commands.spawn_empty()
        .insert((
            Mesh3d::from(mesh.clone()),
            MeshMaterial3d::from(material.clone()),
        ))
        .insert(bpx::RigidBody::Dynamic)
        .insert(bpx::Shape {
            material: px_material,
            geometry: px_geometry,
            ..default()
        })
        .insert(Kinematic::new(transform))
        .insert(PlayerControlled)
        .insert(Name::new("Cue"));
}

fn move_kinematic(
    scene: Res<bpx::Scene>,
    window: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    surface: Query<Entity, With<Surface>>,
    mut kinematic: Query<&mut Kinematic, With<PlayerControlled>>,
) {
    let Ok(window) = window.get_single() else { return; };
    let Some(cursor_position) = window.cursor_position() else { return; };
    let Ok(surface_entity) = surface.get_single() else { return; };
    let Ok(mut kinematic) = kinematic.get_single_mut() else { return; };

    for (camera, camera_transform) in &cameras {
        let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position).ok() else { continue; };

        unsafe extern "C" fn raycast_filter(
            actor: *const PxRigidActor,
            _data: *const PxFilterData,
            _shape: *const PxShape,
            _hit_flags: u32,
            user_data: *const c_void,
        ) -> PxQueryHitType {
            let entity = *(user_data as *const Entity);
            if get_actor_entity_from_ptr(actor) == entity {
                PxQueryHitType::Block
            } else {
                PxQueryHitType::None
            }
        }

        let filter = SceneQueryFilter::callback(raycast_filter, &surface_entity as *const Entity as *mut c_void);

        if let Some(hit) = scene.raycast(ray, f32::MAX, &filter) {
            kinematic.target.translation.x = hit.position.x;
            kinematic.target.translation.z = hit.position.z;
            kinematic.target.translation.y = BALL_SIZE;
            kinematic.target.rotation = default();
        }
    }
}

fn spawn_camera_and_light(mut commands: Commands) {
    commands
        .spawn((
            Name::new("Camera"),
            Transform::from_xyz(0., 0., 0.),
            Visibility::default(),
        ))
        .with_children(|builder| {
            builder.spawn((
                Camera3d::default(),
                Transform::from_translation(Vec3::new(-11., 55., 0.)).looking_at(Vec3::ZERO, Vec3::Y),
            ));
        });

    commands.spawn((
        Name::new("Light"), 
        DirectionalLight::default(),
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -1.2, -0.2, 0.)),
    ));
}

