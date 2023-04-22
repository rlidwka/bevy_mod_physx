use crate::components::ArticulationRootHandle;
use crate::prelude::{Scene, *};
use bevy::prelude::*;
use physx::prelude::*;

#[derive(Component, Debug, Default, PartialEq, Reflect, Clone, Copy)]
pub struct ArticulationRoot {
    pub fix_base: bool,
    pub drive_limits_are_forces: bool,
    pub disable_self_collision: bool,
    pub compute_joint_forces: bool,
}

pub struct ArticulationPlugin;

impl Plugin for ArticulationPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ArticulationRoot>();
        app.add_system(articulation_root_sync.in_base_set(PhysicsSet::Sync).in_schedule(PhysicsSchedule));
    }
}

pub fn articulation_root_sync(
    mut scene: ResMut<Scene>,
    mut actors: Query<(Option<&mut ArticulationRootHandle>, &ArticulationRoot), Changed<ArticulationRoot>>
) {
    // this function only applies user defined properties,
    // there's nothing to get back from physx engine
    for (root, flags) in actors.iter_mut() {
        if let Some(mut root) = root {
            let mut handle = root.get_mut(&mut scene);
            handle.set_articulation_flag(ArticulationFlag::FixBase, flags.fix_base);
            handle.set_articulation_flag(ArticulationFlag::DriveLimitsAreForces, flags.drive_limits_are_forces);
            handle.set_articulation_flag(ArticulationFlag::DisableSelfCollision, flags.disable_self_collision);
            handle.set_articulation_flag(ArticulationFlag::ComputeJointForces, flags.compute_joint_forces);
        } else {
            bevy::log::warn!("ArticulationBase component exists, but it's not an articulation root");
        };
    }
}
