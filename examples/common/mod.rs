// Set of tools to make demos more useful.
//  - configure environment (ambient light, shadows, adjust light settings)
//  - orbit camera controller (click+drag to rotate, right click+drag to pan, scroll to zoom)
//  - bevy inspector (F12 to toggle)
//  - debug lines (F11 to toggle)
//  - press ESC to close demo
//
// This module is intended to be optional, all the stuff should work without it.
//
const SIMULATION_STARTS_PAUSED: bool = false;
const INSPECTOR_STARTS_HIDDEN: bool = false;

use std::ffi::CString;
use std::time::Duration;

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::input::common_conditions::input_toggle_active;
use bevy::pbr::DirectionalLightShadowMap;
use bevy::prelude::*;
use bevy_mod_physx::prelude::*;

pub mod debug_render;
use debug_render::DebugRenderPlugin;

pub mod orbit_camera;
use orbit_camera::{OrbitCamera, OrbitCameraPlugin};

#[derive(Default)]
pub struct DemoUtils;

impl Plugin for DemoUtils {
    fn build(&self, app: &mut App) {
        app.insert_resource(NameFormatter(|entity, name| {
            // set custom name in PVD
            let str = if let Some(name) = name {
                format!("{name} ({entity:?})")
            } else {
                format!("({entity:?})")
            };

            std::borrow::Cow::Owned(CString::new(str).unwrap())
        }));
        app.add_plugins(DebugRenderPlugin);

        app.insert_resource(ClearColor(Color::rgb(0., 0., 0.)));
        app.insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0 / 5.0f32,
        });
        app.insert_resource(Msaa::default());
        app.insert_resource(DirectionalLightShadowMap { size: 4096 });

        // log fps to console
        app.add_plugins(FrameTimeDiagnosticsPlugin);
        app.add_plugins(LogDiagnosticsPlugin {
            wait_duration: Duration::from_millis(1000),
            filter: Some(vec![FrameTimeDiagnosticsPlugin::FPS]),
            ..default()
        });

        app.add_plugins(OrbitCameraPlugin);
        app.add_plugins(
            bevy_inspector_egui::quick::WorldInspectorPlugin::default()
                .run_if(input_toggle_active(!INSPECTOR_STARTS_HIDDEN, KeyCode::F12)),
        );
        app.add_systems(Update, adjust_light_settings);
        app.add_systems(Update, adjust_camera_settings);
        app.add_systems(Update, spacebar_pauses_simulation);

        if SIMULATION_STARTS_PAUSED {
            app.add_systems(Startup, |mut time: ResMut<Time>| time.pause());
        }

        app.add_systems(Update, bevy::window::close_on_esc);
    }
}

#[derive(Component)]
struct DemoUtilsLightFixed;

fn adjust_light_settings(
    mut commands: Commands,
    mut query: Query<(Entity, &mut DirectionalLight), Without<DemoUtilsLightFixed>>,
) {
    // We don't want to copy-paste this code into all examples,
    // but we still want examples to be functional by default without this plugin.
    // So a good solution is to get existing light and adjust it to look nicer.
    for (entity, mut light) in query.iter_mut() {
        light.illuminance = 15000.;
        light.shadows_enabled = true;
        commands.entity(entity).insert(DemoUtilsLightFixed);
    }
}

#[allow(clippy::type_complexity)]
fn adjust_camera_settings(
    mut commands: Commands,
    mut query: Query<(Entity, &Transform), (With<Camera3d>, Without<OrbitCamera>)>,
) {
    // Attach orbit camera controller to the existing camera.
    for (entity, transform) in query.iter_mut() {
        let (yaw, pitch, _roll) = transform.rotation.to_euler(EulerRot::YXZ);

        commands.entity(entity)
            .insert(OrbitCamera {
                gimbal_x: -yaw,
                gimbal_y: -pitch,
                distance: transform.translation.length(),
                ..default()
            });
    }
}

fn spacebar_pauses_simulation(
    keys: Res<Input<KeyCode>>,
    mut time: ResMut<Time>,
) {
    if keys.just_pressed(KeyCode::Space) {
        if time.is_paused() {
            time.unpause();
        } else {
            time.pause();
        }
    }
}
