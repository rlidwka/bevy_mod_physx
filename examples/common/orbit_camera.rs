// flying camera that you can control with mouse, I still didn't find a good crate for it
// maybe switch to smooth-bevy-cameras, but still needs a custom controller

use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::EguiContexts;
use std::f32::consts::PI;

pub struct OrbitCameraPlugin;

impl Plugin for OrbitCameraPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<OrbitCamera>();
        app.add_system(apply_camera_controls);
        app.add_system(update_camera.after(apply_camera_controls));
    }
}

#[derive(Bundle, Default)]
pub struct OrbitCameraBundle {
    pub orbit_camera: OrbitCamera,
    pub camera3d: Camera3dBundle,
}

#[derive(Debug, Component, Reflect)]
pub struct OrbitCamera {
    pub zoom_sensitivity: f32,
    pub rotate_sensitivity: f32,
    pub pan_sensitivity: f32,
    pub gimbal_x: f32,
    pub gimbal_y: f32,
    pub distance: f32,
    pub min_distance: f32,
    pub max_distance: f32,
    pub min_y_angle: f32,
    pub max_y_angle: f32,
    pub target: Vec3,
    pub active: bool,
    pub last_rotation: Quat,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            zoom_sensitivity: 0.1,
            rotate_sensitivity: 0.003,
            pan_sensitivity: 0.001,
            gimbal_x: 60f32.to_radians(),
            gimbal_y: 20f32.to_radians(),
            distance: 3.,
            min_distance: 0.,
            max_distance: f32::INFINITY,
            min_y_angle: 0.02,
            max_y_angle: PI / 2.2,
            target: Vec3::ZERO,
            active: true,
            last_rotation: Quat::IDENTITY,
        }
    }
}

fn apply_camera_controls(
    mut scroll_events: EventReader<MouseWheel>,
    mut move_events: EventReader<MouseMotion>,
    buttons: Res<Input<MouseButton>>,
    mut egui_contexts: EguiContexts,
    mut camera_query: Query<&mut OrbitCamera>,
) {
    let egui_ctx = egui_contexts.ctx_mut();
    if egui_ctx.wants_pointer_input() { return; }

    enum MyEvent {
        Zoom(f32),
        Rotate((f32, f32)),
        Pan((f32, f32)),
    }

    let mut events = vec![];

    for ev in scroll_events.iter() {
        events.push(MyEvent::Zoom(ev.y));
    }

    if buttons.pressed(MouseButton::Left) {
        for ev in move_events.iter() {
            events.push(MyEvent::Rotate((ev.delta.x, ev.delta.y)));
        }
    } else if buttons.pressed(MouseButton::Right) {
        for ev in move_events.iter() {
            events.push(MyEvent::Pan((ev.delta.x, ev.delta.y)));
        }
    }

    if events.is_empty() { return; }

    let mut camcount = 0;
    for mut camera in camera_query.iter_mut() {
        if !camera.active { return; }
        camcount += 1;

        for event in events.iter() {
            match event {
                MyEvent::Zoom(dy) => {
                    camera.distance = (camera.distance * ((1. + camera.zoom_sensitivity).powf(-dy)))
                        .clamp(camera.min_distance, camera.max_distance);
                }
                MyEvent::Rotate((dx, dy)) => {
                    camera.gimbal_x += dx * camera.rotate_sensitivity;
                    camera.gimbal_y = (camera.gimbal_y + dy * camera.rotate_sensitivity)
                        .clamp(camera.min_y_angle, camera.max_y_angle);
                }
                MyEvent::Pan((dx, dy)) => {
                    let v = Vec2::new(*dx, *dy).rotate(-Vec2::from_angle(camera.gimbal_x));
                    camera.target.x += v.x * camera.pan_sensitivity * camera.distance;
                    camera.target.z += v.y * camera.pan_sensitivity * camera.distance;
                }
            }
        }
    }

    if camcount > 1 {
        bevy::log::warn!("found {} active FlyingCameras, only 1 expected", camcount);
    }
}

fn update_camera(
    mut commands: Commands,
    mut camera_query: Query<(Entity, &mut OrbitCamera)>,
    time: Res<Time>,
) {
    let delta = time.delta_seconds();
    let focus_rotation = Quat::IDENTITY;

    for (entity, mut camera) in camera_query.iter_mut() {
        if !camera.active { return; }

        camera.last_rotation = focus_rotation.slerp(camera.last_rotation, 1. - delta * 10.);

        let quat = Quat::from_euler(EulerRot::YXZ, -camera.gimbal_x, -camera.gimbal_y, 0.);

        let mut new_transform = Transform::from_translation(
            camera.target +
            (camera.last_rotation * quat * Vec3::Z) * camera.distance
        );

        new_transform.look_at(camera.target, camera.last_rotation * Vec3::Y);

        commands.entity(entity).insert(new_transform);
    }
}
