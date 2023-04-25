// Set of tools to make demos more useful.
//  - configure environment (ambient light, shadows, adjust light settings)
//  - orbit camera controller (click+drag to rotate, right click+drag to pan, scroll to zoom)
//  - bevy inspector (F12 to toggle)
//  - debug lines (F11 to toggle)
//  - press ESC to close demo
//
// This module is intended to be optional, all the stuff should work without it.
//

use bevy::{prelude::*, pbr::DirectionalLightShadowMap, input::common_conditions::input_toggle_active};

pub mod debug_lines;
use debug_lines::DebugLinesPlugin;

pub mod orbit_camera;
use orbit_camera::{OrbitCamera, OrbitCameraPlugin};

#[derive(Default)]
pub struct DemoUtils;

impl Plugin for DemoUtils {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::rgb(0., 0., 0.)));
        app.insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0 / 5.0f32,
        });
        app.insert_resource(Msaa::default());
        app.insert_resource(DirectionalLightShadowMap { size: 4096 });

        app.add_plugin(OrbitCameraPlugin);
        app.add_plugin(DebugLinesPlugin);
        app.add_plugin(
            bevy_inspector_egui::quick::WorldInspectorPlugin::default()
                .run_if(input_toggle_active(true, KeyCode::F12))
        );
        app.add_system(adjust_light_settings);
        app.add_system(adjust_camera_settings);
        app.add_system(bevy::window::close_on_esc);
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
