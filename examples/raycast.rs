// ported from ray_casting3 example from bevy_rapier3d
mod flying_camera;

use bevy::pbr::DirectionalLightShadowMap;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use flying_camera::*;

use bevy_physx::prelude::*;
use bevy_physx::prelude as bpx;

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
        .add_plugin(PhysXPlugin::default())
        .add_plugin(PhysXDebugRenderPlugin)
        .add_plugin(FlyingCameraPlugin)
        .add_startup_systems((
            init_materials,
            apply_system_buffers,
            spawn_light,
            spawn_camera,
            spawn_plane,
            spawn_cubes,
        ).chain())
        .add_systems((
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

fn spawn_cubes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<DemoMaterials>,
    mut px_geometries: ResMut<Assets<bpx::Geometry>>,
) {
    let num = 8;
    let rad = 1.0;
    let px_geometry = px_geometries.add(bpx::Geometry::cuboid(rad, rad, rad));
    let mesh = meshes.add(Mesh::from(shape::Cube { size: rad * 2. }));
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
                    PbrBundle {
                        mesh: mesh.clone(),
                        material: material.clone(),
                        transform: Transform::from_xyz(x, y, z),
                        ..default()
                    },
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
            .insert(materials.normal.clone())
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
    let Ok(window) = window.get_single() else { return; };
    let Some(cursor_position) = window.cursor_position() else { return; };

    for (camera, camera_transform) in &cameras {
        let Some(ray) = camera.viewport_to_world(camera_transform, cursor_position) else { continue; };

        if let Some(hit) = scene.raycast(ray.origin, ray.direction, f32::MAX) {
            if highlighable.get(hit.actor).is_ok() {
                commands.entity(hit.actor)
                    .insert(materials.highlighted.clone())
                    .insert(Highlighted);
            }
        }
    }
}
