use std::f32::consts::PI;

use bevy::prelude::*;
use bevy::input::mouse::{MouseWheel, MouseMotion};

pub struct FlyingCameraPlugin;

impl Plugin for FlyingCameraPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<FlyingCamera>();
        app.add_system(apply_camera_controls);
        app.add_system(update_camera.after(apply_camera_controls));
    }
}

#[derive(Bundle, Default)]
pub struct FlyingCameraBundle {
    pub flying_camera: FlyingCamera,
    pub camera3d: Camera3dBundle,
}

#[derive(Debug, Component, Reflect)]
pub struct FlyingCamera {
    pub zoom_sensitivity: f32,
    pub rotate_sensitivity: f32,
    pub gimbal_x: f32,
    pub gimbal_y: f32,
    pub distance: f32,
    pub min_distance: f32,
    pub max_distance: f32,
    pub min_y_angle: f32,
    pub max_y_angle: f32,
    pub active: bool,
    pub last_rotation: Quat,
}

impl Default for FlyingCamera {
    fn default() -> Self {
        Self {
            zoom_sensitivity: 0.1,
            rotate_sensitivity: 0.003,
            gimbal_x: -60f32.to_radians(),
            gimbal_y: -20f32.to_radians(),
            distance: 3.,
            min_distance: 0.,
            max_distance: f32::INFINITY,
            min_y_angle: 0.02,
            max_y_angle: PI / 2.2,
            active: true,
            last_rotation: Quat::IDENTITY,
        }
    }
}

fn apply_camera_controls(
    mut scroll_events: EventReader<MouseWheel>,
    mut move_events: EventReader<MouseMotion>,
    buttons: Res<Input<MouseButton>>,
    mut camera_query: Query<&mut FlyingCamera>,
) {
    enum MyEvent {
        Zoom(f32),
        Rotate((f32, f32)),
    }

    let mut events = vec![];

    for ev in scroll_events.iter() {
        events.push(MyEvent::Zoom(ev.y));
    }

    if buttons.pressed(MouseButton::Left) {
        for ev in move_events.iter() {
            events.push(MyEvent::Rotate((ev.delta.x, ev.delta.y)));
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
                    camera.gimbal_x -= dx * camera.rotate_sensitivity;
                    camera.gimbal_y = (camera.gimbal_y - dy * camera.rotate_sensitivity)
                        .clamp(-camera.max_y_angle, -camera.min_y_angle);
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
    mut camera_query: Query<(Entity, &mut FlyingCamera)>,
    time: Res<Time>,
) {
    let delta = time.delta_seconds();

    let focus_position = Vec3::ZERO;
    let focus_rotation = Quat::IDENTITY;

    for (entity, mut camera) in camera_query.iter_mut() {
        if !camera.active { return; }

        camera.last_rotation = focus_rotation.slerp(camera.last_rotation, 1. - delta * 10.);

        let quat = Quat::from_euler(EulerRot::YXZ, camera.gimbal_x, camera.gimbal_y, 0.);

        let mut new_transform = Transform::from_translation(
            focus_position +
            (camera.last_rotation * quat * Vec3::Z) * camera.distance
        );

        new_transform.look_at(focus_position, camera.last_rotation * Vec3::Y);

        commands.entity(entity).insert(new_transform);
    }
}
