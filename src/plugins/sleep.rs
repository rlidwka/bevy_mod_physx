//! Two-way sync of sleep state, wake counter, sleep threshold, etc.
use std::sync::mpsc::channel;

use bevy::prelude::*;
use physx::prelude::*;
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

use crate::types::OnWakeSleep;
use crate::prelude::{Scene, *};

#[derive(Component, Debug, Default, PartialEq, Eq, Clone, Copy, Hash, Reflect)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[reflect(Component, Default)]
#[component(storage = "SparseSet")]
/// Marker component added to sleeping entities.
///
/// Do not add or remove this component. If you want to put to sleep or
/// wake up an actor, use [SleepControl] instead.
pub struct Sleeping;

#[derive(Resource)]
pub(crate) struct WakeSleepCallback(pub(crate) OnWakeSleep);

#[derive(Event)]
pub struct WakeSleepEvent {
    pub entities: Vec<Entity>,
    pub is_waking: bool,
}

#[derive(Component, Debug, Default, PartialEq, Clone, Copy, Reflect)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[reflect(Component, Default)]
/// Set sleep state, sleep threshold, stabilization threshold, wake counter.
pub struct SleepControl {
    /// Whether this actor is sleeping or not.
    pub is_sleeping: bool,
    /// Mass-normalized kinetic energy below which an actor may go to sleep.
    pub sleep_threshold: f32,
    /// Mass-normalized kinetic energy threshold below which an actor
    /// may participate in stabilization.
    pub stabilization_threshold: f32,
    /// Wake counter for the actor.
    ///
    /// The wake counter value determines the minimum amount of time until
    /// the body can be put to sleep. Please note that a body will not be put
    /// to sleep if the energy is above the specified threshold (`sleep_threshold`)
    /// or if other awake bodies are touching it.
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

pub struct SleepPlugin;

impl Plugin for SleepPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Sleeping>();
        app.register_type::<SleepControl>();

        // note: system is added in Create set, not in Sync, because it adds/removes
        // component as opposed to modifying it.
        app.add_systems(PhysicsSchedule, sleep_marker_sync.in_set(PhysicsSet::Create));
        app.add_systems(PhysicsSchedule, sleep_control_sync.in_set(PhysicsSet::Sync));

        let (wake_sleep_sender, wake_sleep_receiver) = channel();

        app.world.insert_resource(WakeSleepCallback(OnWakeSleep::new(move |actors, is_waking| {
            let entities = actors.iter().map(|actor| {
                actor.cast_map(
                    |articulation| *articulation.get_user_data(),
                    |rstatic| *rstatic.get_user_data(),
                    |rdynamic| *rdynamic.get_user_data(),
                )
            }).collect::<Vec<_>>();

            wake_sleep_sender.send(WakeSleepEvent { entities, is_waking }).unwrap();
        })));

        app.add_physics_event_channel(wake_sleep_receiver);
    }

    fn cleanup(&self, app: &mut App) {
        // Resource shall be consumed when creating physics scene.
        // If it doesn't, it means sleep plugin is loaded after scene is created,
        // which shouldn't happen.
        assert!(!app.world.contains_resource::<WakeSleepCallback>());
    }
}

pub fn sleep_marker_sync(
    mut commands: Commands,
    mut events: EventReader<WakeSleepEvent>,
) {
    for WakeSleepEvent { entities, is_waking } in events.iter() {
        for entity in entities.iter() {
            let Some(mut cmd) = commands.get_entity(*entity) else { continue; };
            if *is_waking {
                cmd.remove::<Sleeping>();
            } else {
                cmd.insert(Sleeping);
            }
        }
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
