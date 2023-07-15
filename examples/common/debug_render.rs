use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::EguiContexts;
use bevy_inspector_egui::egui::{self, Align2, Window};
use bevy_physx::render::DebugRenderSettings;

pub struct DebugRenderPlugin;

impl Plugin for DebugRenderPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, debug_render_toggle);
        app.add_systems(Update, display_debug_render_ui);

        app.insert_resource(DebugRenderSettings {
            scale: 0., // global toggle for everything
            ..DebugRenderSettings::enable()
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

fn display_debug_render_ui(
    mut egui_contexts: EguiContexts,
    mut settings: ResMut<DebugRenderSettings>,
) {
    const TOOLTIP_MARGIN: f32 = 10.;
    let ctx = egui_contexts.ctx_mut();
    Window::new("debug_render_ui")
        .anchor(Align2::RIGHT_TOP, egui::vec2(-TOOLTIP_MARGIN, TOOLTIP_MARGIN))
        .title_bar(false)
        .auto_sized()
        .resizable(false)
        .show(ctx, |ui| {
            macro_rules! checkbox {
                ($setting: ident, $desc: literal) => {
                    let orig_value = settings.$setting > 0.;
                    let mut value = orig_value;
                    ui.checkbox(&mut value, $desc);
                    if orig_value != value {
                        settings.$setting = if value { 1. } else { 0. };
                    }
                };
            }

            checkbox!(scale, "Debug Render");

            if settings.scale > 0. {
                checkbox!(world_axes, "world_axes");
                checkbox!(body_axes, "body_axes");
                checkbox!(body_mass_axes, "body_mass_axes");
                checkbox!(body_lin_velocity, "body_lin_velocity");
                checkbox!(body_ang_velocity, "body_ang_velocity");
                checkbox!(contact_point, "contact_point");
                checkbox!(contact_normal, "contact_normal");
                checkbox!(contact_error, "contact_error");
                checkbox!(contact_force, "contact_force");
                checkbox!(actor_axes, "actor_axes");
                checkbox!(collision_aabbs, "collision_aabbs");
                checkbox!(collision_shapes, "collision_shapes");
                checkbox!(collision_axes, "collision_axes");
                checkbox!(collision_compounds, "collision_compounds");
                checkbox!(collision_fnormals, "collision_fnormals");
                checkbox!(collision_edges, "collision_edges");
                checkbox!(collision_static, "collision_static");
                checkbox!(collision_dynamic, "collision_dynamic");
                checkbox!(joint_local_frames, "joint_local_frames");
                checkbox!(joint_limits, "joint_limits");
                checkbox!(cull_box, "cull_box");
                checkbox!(mbp_regions, "mbp_regions");
                checkbox!(simulation_mesh, "simulation_mesh");
                checkbox!(sdf, "sdf");
            }
        });
}
