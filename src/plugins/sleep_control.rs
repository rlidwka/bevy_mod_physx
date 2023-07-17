use bevy::prelude::*;
use physx::rigid_dynamic::RigidDynamic;

use crate::components::{ArticulationRootHandle, RigidDynamicHandle};
use crate::prelude::{Scene, *};

#[derive(Component, Debug, Default, PartialEq, Clone, Copy, Reflect)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[reflect(Component, Default)]
pub struct SleepControl {
    pub is_sleeping: bool,
    pub sleep_threshold: f32,
    pub stabilization_threshold: f32,
    pub wake_counter: f32,
}

impl SleepControl {
    pub fn put_to_sleep(&mut self) {
        self.is_sleeping = true;
    }

    pub fn wake_up(&mut self) {
        self.is_sleeping = false;
    }
}

pub struct SleepControlPlugin;

impl Plugin for SleepControlPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<SleepControl>();
        app.add_systems(PhysicsSchedule, sleep_control_sync.in_set(PhysicsSet::Sync));
    }
}

pub fn sleep_control_sync(
    mut scene: ResMut<Scene>,
    mut actors: Query<(
        Option<&mut RigidDynamicHandle>,
        Option<&mut ArticulationRootHandle>,
        &mut SleepControl,
    ), Or<(Changed<SleepControl>, Without<Sleeping>)>>,
) {
    // this function does two things: sets physx property (if changed) or writes it back (if not);
    // we need it to happen inside a single system to avoid change detection loops, but
    // user will experience 1-tick delay on any changes
    for (dynamic, _articulation_base, mut control) in actors.iter_mut() {
        if let Some(mut actor) = dynamic {
            if control.is_changed() || (actor.is_added() && *control != default()) {
                let mut handle = actor.get_mut(&mut scene);

                if control.is_sleeping != handle.is_sleeping() {
                    if control.is_sleeping {
                        handle.put_to_sleep();
                    } else {
                        handle.wake_up();
                    }
                }

                handle.set_stabilization_threshold(control.stabilization_threshold);
                handle.set_sleep_threshold(control.sleep_threshold);
                handle.set_wake_counter(control.wake_counter);
            } else {
                let handle = actor.get(&scene);

                let new_control = SleepControl {
                    is_sleeping: handle.is_sleeping(),
                    sleep_threshold: handle.get_sleep_threshold(),
                    stabilization_threshold: handle.get_stabilization_threshold(),
                    wake_counter: handle.get_wake_counter(),
                };

                // extra check so we don't mutate on every frame without changes
                if *control != new_control { *control = new_control; }
            }
        } else if !control.is_added() {
            bevy::log::warn!("SleepControl component exists, but it's neither a rigid dynamic nor articulation root");
        }
    }
}
