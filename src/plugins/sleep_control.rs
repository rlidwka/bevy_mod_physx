use bevy::prelude::*;
use physx::rigid_dynamic::RigidDynamic;
use physx::traits::Class;
use physx_sys::{
    PxArticulationReducedCoordinate_getSleepThreshold,
    PxArticulationReducedCoordinate_getStabilizationThreshold,
    PxArticulationReducedCoordinate_getWakeCounter,
    PxArticulationReducedCoordinate_isSleeping,
    PxArticulationReducedCoordinate_putToSleep_mut,
    PxArticulationReducedCoordinate_setSleepThreshold_mut,
    PxArticulationReducedCoordinate_setStabilizationThreshold_mut,
    PxArticulationReducedCoordinate_setWakeCounter_mut,
    PxArticulationReducedCoordinate_wakeUp_mut,
};

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
    for (dynamic, articulation_base, mut control) in actors.iter_mut() {
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
        } else if let Some(mut root) = articulation_base {
            if control.is_changed() || (root.is_added() && *control != default()) {
                let mut handle = root.get_mut(&mut scene);

                if control.is_sleeping != unsafe { PxArticulationReducedCoordinate_isSleeping(handle.as_ptr()) } {
                    if control.is_sleeping {
                        unsafe { PxArticulationReducedCoordinate_putToSleep_mut(handle.as_mut_ptr()) };
                    } else {
                        unsafe { PxArticulationReducedCoordinate_wakeUp_mut(handle.as_mut_ptr()) };
                    }
                }

                unsafe {
                    PxArticulationReducedCoordinate_setStabilizationThreshold_mut(handle.as_mut_ptr(), control.stabilization_threshold);
                    PxArticulationReducedCoordinate_setSleepThreshold_mut(handle.as_mut_ptr(), control.sleep_threshold);
                    PxArticulationReducedCoordinate_setWakeCounter_mut(handle.as_mut_ptr(), control.wake_counter);
                }
            } else {
                let handle = root.get(&scene);

                let new_control = SleepControl {
                    is_sleeping: unsafe { PxArticulationReducedCoordinate_isSleeping(handle.as_ptr()) },
                    sleep_threshold: unsafe { PxArticulationReducedCoordinate_getSleepThreshold(handle.as_ptr()) },
                    stabilization_threshold: unsafe { PxArticulationReducedCoordinate_getStabilizationThreshold(handle.as_ptr()) },
                    wake_counter: unsafe { PxArticulationReducedCoordinate_getWakeCounter(handle.as_ptr()) },
                };

                // extra check so we don't mutate on every frame without changes
                if *control != new_control { *control = new_control; }
            }
        } else if !control.is_added() {
            bevy::log::warn!("SleepControl component exists, but it's neither a rigid dynamic nor articulation root");
        }
    }
}
