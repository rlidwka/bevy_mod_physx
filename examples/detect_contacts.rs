mod common;

use std::sync::mpsc::channel;

use bevy::prelude::*;
use bevy_mod_physx::callbacks::OnCollision;
use bevy_mod_physx::prelude::{self as bpx, *};
use bevy_mod_physx::utils::{get_actor_entity_from_ptr, get_shape_entity_from_ptr};
use physx::scene::FilterShaderDescriptor;
use physx_sys::{
    FilterShaderCallbackInfo,
    PxContactPairFlags,
    PxContactPair_extractContacts,
    PxFilterFlags,
    PxPairFlags,
};

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

unsafe extern "C" fn simulation_filter_shader(s: *mut FilterShaderCallbackInfo) -> PxFilterFlags {
    let s = &mut *s as &mut FilterShaderCallbackInfo;
    let pair_flags = &mut *(s.pairFlags) as &mut PxPairFlags;

    *pair_flags = PxPairFlags::SolveContact |
        PxPairFlags::DetectDiscreteContact |
        PxPairFlags::NotifyTouchFound |
        PxPairFlags::NotifyTouchLost |
        PxPairFlags::NotifyContactPoints;

    PxFilterFlags::empty()
}

#[derive(Event)]
pub struct CollisionEvent {
    actor0: Entity,
    actor1: Entity,
}

fn main() {
    let (mpsc_sender, mpsc_receiver) = channel();

    let on_collision = OnCollision::new(move |header, pairs| {
        assert!(!header.flags.contains(physx_sys::PxContactPairHeaderFlags::RemovedActor0));
        let actor0 = unsafe { get_actor_entity_from_ptr(header.actors[0] as *const _) };

        assert!(!header.flags.contains(physx_sys::PxContactPairHeaderFlags::RemovedActor1));
        let actor1 = unsafe { get_actor_entity_from_ptr(header.actors[1] as *const _) };

        for pair in pairs {
            // this example shows how to extract contact details
            assert!(!pair.flags.contains(PxContactPairFlags::RemovedShape0));
            let shape0 = unsafe { get_shape_entity_from_ptr(pair.shapes[0] as *const _) };

            assert!(!pair.flags.contains(PxContactPairFlags::RemovedShape0));
            let shape1 = unsafe { get_shape_entity_from_ptr(pair.shapes[1] as *const _) };

            let contacts = unsafe {
                let mut buffer = Vec::with_capacity(pairs.len());
                PxContactPair_extractContacts(pair, buffer.as_mut_ptr(), pairs.len() as u32);
                buffer.set_len(pairs.len());
                buffer
            };

            let mut status = "Contact";
            if pair.flags.contains(PxContactPairFlags::ActorPairHasFirstTouch) {
                status = "New contact";
            }
            if pair.flags.contains(PxContactPairFlags::ActorPairLostTouch) {
                status = "Lost contact";
            }

            println!("{status} between {shape0:?} and {shape1:?}:");
            for contact in contacts {
                println!(
                    "position: {:?}, separation: {:?}, normal: {:?}, impulse: {:?}",
                    contact.position.to_bevy(),
                    contact.separation,
                    contact.normal.to_bevy(),
                    contact.impulse.to_bevy(),
                );
            }
            println!("----------");
        }

        // this callback must not do anything with the scene,
        // we need to extract required data and send it through a channel into
        // a system that handles things
        mpsc_sender.send(CollisionEvent { actor0, actor1 }).unwrap();
    });

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PhysicsPlugins.set(PhysicsCore {
            scene: bpx::SceneDescriptor {
                // simulation filter shader will filter details that we get in on_collision callback,
                // by default on_collision callback doesn't do anything
                simulation_filter_shader: FilterShaderDescriptor::CallDefaultFirst(simulation_filter_shader),

                // callback is a closure, where we pass Sender to
                on_collision: Some(on_collision),
                ..default()
            },
            ..default()
        }))
        .add_plugins(common::DemoUtils) // optional
        .add_physics_event_channel(mpsc_receiver)
        .add_systems(Startup, (
            init_materials,
            apply_deferred,
            (
                spawn_plane,
                spawn_tiles,
                spawn_dynamic,
                spawn_camera_and_light,
            ),
        ).chain())
        .add_systems(Update, highlight_on_hit)
        .run();
}

fn init_materials(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.insert_resource(DemoMaterials {
        normal: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
        highlighted: materials.add(Color::rgb(0.3, 0.4, 0.9).into()),
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
    let mesh = meshes.add(Mesh::from(shape::Plane { size: 500.0, subdivisions: 4 }));
    let material = materials.add(Color::rgb(0.3, 0.5, 0.3).into());
    let px_geometry = px_geometries.add(bpx::Geometry::halfspace(Vec3::Y));
    let px_material = px_materials.add(bpx::Material::new(&mut physics, 0.5, 0.5, 0.6));

    commands.spawn_empty()
        .insert(PbrBundle {
            mesh,
            material,
            ..default()
        })
        .insert(bpx::RigidBody::Static)
        .insert(bpx::Shape {
            geometry: px_geometry,
            material: px_material,
            ..default()
        })
        .insert(Name::new("Plane"));
}

fn spawn_tiles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<DemoMaterials>,
    mut px_geometries: ResMut<Assets<bpx::Geometry>>,
) {
    let num = 32;
    let rad = 1.0;
    let height = 0.1;
    let px_geometry = px_geometries.add(bpx::Geometry::cuboid(rad, height, rad));
    let mesh = meshes.add(Mesh::from(shape::Box { min_x: -rad, min_y: -height, min_z: -rad, max_x: rad, max_y: height, max_z: rad }));
    let material = materials.normal.clone();

    let shift = rad * 2.5;
    let centerx = shift * (num / 2) as f32;
    let centerz = shift * (num / 2) as f32;

    for i in 0..num {
        for j in 0..num {
            let x = i as f32 * shift - centerx;
            let y = height / 2.;
            let z = j as f32 * shift - centerz;

            commands.spawn((
                PbrBundle {
                    mesh: mesh.clone(),
                    material: material.clone(),
                    transform: Transform::from_xyz(x, y, z),
                    ..default()
                },
                RigidBody::Static,
                bpx::Shape {
                    geometry: px_geometry.clone(),
                    ..default()
                },
                Highlightable,
            ));
        }
    }
}

fn spawn_dynamic(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut physics: ResMut<Physics>,
    mut px_geometries: ResMut<Assets<bpx::Geometry>>,
    mut px_materials: ResMut<Assets<bpx::Material>>,
) {
    const RADIUS: f32 = 1.25;

    let mesh = meshes.add(Mesh::from(shape::UVSphere { radius: 1.25, ..default() }));
    let material = materials.add(Color::rgb(0.8, 0.7, 0.6).into());

    let px_geometry = px_geometries.add(bpx::Geometry::ball(RADIUS));
    let px_material = px_materials.add(bpx::Material::new(&mut physics, 0., 0., 1.));

    let transform = Transform::from_xyz(0., 5., 32.5);

    commands.spawn_empty()
        .insert(PbrBundle {
            mesh,
            material,
            transform,
            ..default()
        })
        .insert(bpx::RigidBody::Dynamic)
        .insert(MassProperties::density(10.))
        .insert(bpx::Shape {
            material: px_material,
            geometry: px_geometry,
            ..default()
        })
        .insert(Velocity::linear(Vec3::new(2.5, -5., -10.)))
        .insert(Name::new("Ball"));
}

fn spawn_camera_and_light(mut commands: Commands) {
    commands
        .spawn(SpatialBundle::from_transform(Transform::from_xyz(-21., 0., 0.)))
        .with_children(|builder| {
            builder.spawn(Camera3dBundle {
                transform: Transform::from_translation(Vec3::new(-41.7, 33., 0.)).looking_at(Vec3::ZERO, Vec3::Y),
                ..default()
            });
        })
        .insert(Name::new("Camera"));

    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -1.2, -0.2, 0.)),
        ..default()
    }).insert(Name::new("Light"));
}

fn highlight_on_hit(
    mut commands: Commands,
    materials: Res<DemoMaterials>,
    mut events: EventReader<CollisionEvent>,
    highlighable: Query<(), With<Highlightable>>,
) {
    for event in events.iter() {
        for entity in [ event.actor0, event.actor1 ] {
            if highlighable.get(entity).is_ok() {
                commands.entity(entity)
                    .insert(materials.highlighted.clone())
                    .insert(Highlighted);
            }
        }
    }
}
