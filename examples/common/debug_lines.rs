use bevy::prelude::*;
use bevy_physx::render::{PhysXDebugRenderPlugin, DebugRenderSettings};

pub struct DebugLinesPlugin;

impl Plugin for DebugLinesPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<PhysXDebugRenderPlugin>() {
            app.add_plugin(PhysXDebugRenderPlugin);
        }
        app.add_system(debug_lines_toggle);
    }
}

fn debug_lines_toggle(
    input: Res<Input<KeyCode>>,
    mut settings: ResMut<DebugRenderSettings>,
) {
    if input.just_pressed(KeyCode::F11) {
        settings.visibility = match settings.visibility {
            Visibility::Hidden => Visibility::Visible,
            Visibility::Inherited => Visibility::Hidden,
            Visibility::Visible => Visibility::Hidden,
        }
    }
}
