use bevy::prelude::*;
use bevy_physx::render::DebugRenderSettings;

pub struct DebugRenderPlugin;

impl Plugin for DebugRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, debug_render_toggle);
        app.insert_resource(DebugRenderSettings {
            scale: 0., // global toggle for everything
            collision_shapes: 1.,
            joint_limits: 0.5,
            ..default()
        });
    }
}

fn debug_render_toggle(
    input: Res<Input<KeyCode>>,
    mut settings: ResMut<DebugRenderSettings>,
) {
    if input.just_pressed(KeyCode::F11) {
        if settings.scale == 0. {
            settings.scale = 1.;
        } else {
            settings.scale = 0.;
        }
    }
}
