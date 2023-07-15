use std::sync::mpsc::channel;

use bevy::prelude::*;
use physx::prelude::*;

use crate::callbacks::OnWakeSleep;
use crate::physx_extras::ActorMapExtras;
use crate::prelude::*;

#[derive(Component, Debug, Default, PartialEq, Eq, Clone, Copy, Hash, Reflect)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[reflect(Component, Default)]
#[component(storage = "SparseSet")]
pub struct Sleeping;

#[derive(Resource)]
pub struct WakeSleepCallback(pub(crate) OnWakeSleep);

#[derive(Event)]
pub struct WakeSleepEvent {
    pub entities: Vec<Entity>,
    pub is_waking: bool,
}

pub struct SleepPlugin;

impl Plugin for SleepPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Sleeping>();

        // note: system is added in Create set, not in Sync, because it adds/removes
        // component as opposed to modifying it.
        app.add_systems(PhysicsSchedule, sleep_sync.in_set(PhysicsSet::Create));

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

pub fn sleep_sync(
    mut commands: Commands,
    mut events: EventReader<WakeSleepEvent>,
) {
    for WakeSleepEvent { entities, is_waking } in events.iter() {
        for entity in entities.iter() {
            let mut cmd = commands.entity(*entity);
            if *is_waking {
                cmd.remove::<Sleeping>();
            } else {
                cmd.insert(Sleeping);
            }
        }
    }
}
