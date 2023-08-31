//! Two-way sync of sleep state, wake counter, sleep threshold, etc.
use std::sync::mpsc::channel;

use bevy::prelude::*;
use physx::prelude::*;
use physx::rigid_dynamic::RigidDynamic;

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
                actor.cast_map_ref(
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

//
// Stuff that's now present in physx, but we use older version
//
use physx::actor::{ActorMap, ActorType};

pub trait ActorMapExtras<'a, L, S, D>
where
    L: ArticulationLink + 'a,
    S: RigidStatic + 'a,
    D: RigidDynamic + 'a,
{
    fn cast_map_ref<Ret, ALFn, RSFn, RDFn>(
        &'a self,
        articulation_link_fn: ALFn,
        rigid_static_fn: RSFn,
        rigid_dynamic_fn: RDFn,
    ) -> Ret
    where
        ALFn: FnMut(&'a L) -> Ret,
        RSFn: FnMut(&'a S) -> Ret,
        RDFn: FnMut(&'a D) -> Ret;

    fn cast_map_mut<Ret, ALFn, RSFn, RDFn>(
        &'a mut self,
        articulation_link_fn: ALFn,
        rigid_static_fn: RSFn,
        rigid_dynamic_fn: RDFn,
    ) -> Ret
    where
        ALFn: FnMut(&'a mut L) -> Ret,
        RSFn: FnMut(&'a mut S) -> Ret,
        RDFn: FnMut(&'a mut D) -> Ret;

    fn as_rigid_dynamic_ref(&self) -> Option<&D>;
    fn as_rigid_static_ref(&self) -> Option<&S>;
    fn as_articulation_link_ref(&self) -> Option<&L>;

    fn as_rigid_dynamic_mut(&mut self) -> Option<&mut D>;
    fn as_rigid_static_mut(&mut self) -> Option<&mut S>;
    fn as_articulation_link_mut(&mut self) -> Option<&mut L>;
}

impl<'a, L, S, D> ActorMapExtras<'a, L, S, D> for ActorMap<L, S, D>
where
    L: ArticulationLink + 'a,
    S: RigidStatic + 'a,
    D: RigidDynamic + 'a,
{
    fn cast_map_ref<Ret, ALFn, RSFn, RDFn>(
        &'a self,
        mut articulation_link_fn: ALFn,
        mut rigid_static_fn: RSFn,
        mut rigid_dynamic_fn: RDFn,
    ) -> Ret
    where
        ALFn: FnMut(&'a L) -> Ret,
        RSFn: FnMut(&'a S) -> Ret,
        RDFn: FnMut(&'a D) -> Ret,
    {
        // This uses get_type not get_concrete_type because get_concrete_type does not seem to
        // work for actors retrieved via get_active_actors.
        match self.get_type() {
            ActorType::RigidDynamic => {
                rigid_dynamic_fn(unsafe { &*(self as *const _ as *const D) })
            }
            ActorType::RigidStatic => rigid_static_fn(unsafe { &*(self as *const _ as *const S) }),
            ActorType::ArticulationLink => {
                articulation_link_fn(unsafe { &*(self as *const _ as *const L) })
            }
        }
    }

    fn cast_map_mut<Ret, ALFn, RSFn, RDFn>(
        &'a mut self,
        mut articulation_link_fn: ALFn,
        mut rigid_static_fn: RSFn,
        mut rigid_dynamic_fn: RDFn,
    ) -> Ret
    where
        ALFn: FnMut(&'a mut L) -> Ret,
        RSFn: FnMut(&'a mut S) -> Ret,
        RDFn: FnMut(&'a mut D) -> Ret,
    {
        // This uses get_type not get_concrete_type because get_concrete_type does not seem to
        // work for actors retrieved via get_active_actors.
        match self.get_type() {
            ActorType::RigidDynamic => {
                rigid_dynamic_fn(unsafe { &mut *(self as *mut _ as *mut D) })
            }
            ActorType::RigidStatic => rigid_static_fn(unsafe { &mut *(self as *mut _ as *mut S) }),
            ActorType::ArticulationLink => {
                articulation_link_fn(unsafe { &mut *(self as *mut _ as *mut L) })
            }
        }
    }

    /// Tries to cast to RigidDynamic.
    fn as_rigid_dynamic_ref(&self) -> Option<&D> {
        match self.get_type() {
            ActorType::RigidDynamic => unsafe { Some(&*(self as *const _ as *const D)) },
            _ => None,
        }
    }

    /// Tries to cast to RigidStatic.
    fn as_rigid_static_ref(&self) -> Option<&S> {
        match self.get_type() {
            ActorType::RigidStatic => unsafe { Some(&*(self as *const _ as *const S)) },
            _ => None,
        }
    }

    /// Tries to cast to ArticulationLink.
    fn as_articulation_link_ref(&self) -> Option<&L> {
        match self.get_type() {
            ActorType::ArticulationLink => unsafe { Some(&*(self as *const _ as *const L)) },
            _ => None,
        }
    }

    /// Tries to cast to RigidDynamic.
    fn as_rigid_dynamic_mut(&mut self) -> Option<&mut D> {
        match self.get_type() {
            ActorType::RigidDynamic => unsafe { Some(&mut *(self as *mut _ as *mut D)) },
            _ => None,
        }
    }

    /// Tries to cast to RigidStatic.
    fn as_rigid_static_mut(&mut self) -> Option<&mut S> {
        match self.get_type() {
            ActorType::RigidStatic => unsafe { Some(&mut *(self as *mut _ as *mut S)) },
            _ => None,
        }
    }

    /// Tries to cast to ArticulationLink.
    fn as_articulation_link_mut(&mut self) -> Option<&mut L> {
        match self.get_type() {
            ActorType::ArticulationLink => unsafe { Some(&mut *(self as *mut _ as *mut L)) },
            _ => None,
        }
    }
}
