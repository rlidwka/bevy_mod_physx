mod flying_camera;

use std::sync::Mutex;
use std::sync::mpsc::{channel, Receiver};

use bevy::pbr::DirectionalLightShadowMap;
use bevy::prelude::*;
use bevy_physx::utils::{get_actor_entity_from_ptr, get_shape_entity_from_ptr};
use flying_camera::*;

use bevy_physx::{prelude::*, callbacks::OnCollision};
use bevy_physx::prelude as bpx;
use physx::scene::FilterShaderDescriptor;
use physx_sys::{FilterShaderCallbackInfo, PxPairFlags, PxFilterFlags, PxContactPairFlags, PxContactPair_extractContacts};

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

#[derive(Resource)]
pub struct CollisionStream {
    receiver: Mutex<Receiver<CollisionEvent>>,
}

pub struct CollisionEvent {
    actor0: Entity,
    actor1: Entity,
}

fn main() {
    let (tx, rx) = channel();

    App::new()
        .insert_resource(ClearColor(Color::rgb(0., 0., 0.)))
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0 / 5.0f32,
        })
        .insert_resource(Msaa::default())
        .insert_resource(DirectionalLightShadowMap { size: 4096 })
        .add_plugins(DefaultPlugins)
        .add_plugin(bevy_inspector_egui::quick::WorldInspectorPlugin::default())
        .add_system(bevy::window::close_on_esc)
        .insert_resource(CollisionStream { receiver: Mutex::new(rx) })
        .add_plugin(PhysXPlugin {
            scene: bpx::SceneDescriptor {
                // simulation filter shader will filter details that we get in on_collision callback,
                // by default on_collision callback doesn't do anything
                simulation_filter_shader: FilterShaderDescriptor::CallDefaultFirst(simulation_filter_shader),

                // callback is a closure, where we pass Sender to
                on_collision: Some(OnCollision::new(move |header, pairs| {
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
                    tx.send(CollisionEvent { actor0, actor1 }).unwrap();
                })),
                ..default()
            },
            ..default()
        })
        .add_plugin(PhysXDebugRenderPlugin)
        .add_plugin(FlyingCameraPlugin)
        .add_startup_systems((
            init_materials,
            apply_system_buffers,
            spawn_light,
            spawn_camera,
            spawn_plane,
            spawn_tiles,
            spawn_dynamic,
        ).chain())
        .add_system(highlight_on_hit)
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

fn spawn_light(mut commands: Commands) {
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 15000.,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform {
            rotation: Quat::from_euler(EulerRot::XYZ, -1.2, -0.2, 0.),
            ..default()
        },
        ..default()
    })
    .insert(Name::new("Light"));
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(FlyingCameraBundle {
        flying_camera: FlyingCamera {
            distance: 40.,
            ..default()
        },
        ..default()
    })
    .insert(Name::new("Camera"));
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

    let transform = Transform::from_translation(Vec3::new(0., 5., 32.5));

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

fn highlight_on_hit(
    mut commands: Commands,
    materials: Res<DemoMaterials>,
    events: Res<CollisionStream>,
    highlighable: Query<(), With<Highlightable>>,
) {
    let Ok(events) = events.receiver.try_lock() else { return; };

    while let Ok(event) = events.try_recv() {
        for entity in [ event.actor0, event.actor1 ] {
            if highlighable.get(entity).is_ok() {
                commands.entity(entity)
                    .insert(materials.highlighted.clone())
                    .insert(Highlighted);
            }
        }
    }
}
