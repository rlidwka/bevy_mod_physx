// bevy_physx doesn't add any events, this module only helps you with adding your own.
//
// It introduces event channels, on one side of which is mpsc::Sender<T>, and on another
// side is bevy's EventReader<T>, and it automatically bridges between the two.
// This feature is needed to get information from physx callbacks to bevy world.

use bevy::prelude::*;
use std::sync::mpsc::Receiver;
use std::sync::Mutex;

use crate::PhysicsSchedule;

#[derive(Resource, Deref, DerefMut)]
struct ChannelReceiver<T>(Mutex<Receiver<T>>);

pub trait AppExtensions {
    // Manage events of type `T` in physics schedule.
    // Note that user setups may run physics any number of times per frame
    // (more than once or less than once are both possible).
    // Thus, all events must be handled in physics schedule, so they won't be missed.
    fn add_physics_event<T: Event>(&mut self) -> &mut Self;

    // Allows you to create bevy events using mpsc Sender
    fn add_event_channel<T: Event>(&mut self, receiver: Receiver<T>) -> &mut Self;

    // Allows you to create bevy events in physics schedule using mpsc Sender
    fn add_physics_event_channel<T: Event>(&mut self, receiver: Receiver<T>) -> &mut Self;
}

impl AppExtensions for App {
    fn add_physics_event<T: Event>(&mut self) -> &mut Self {
        if !self.world.contains_resource::<Events<T>>() {
            self.init_resource::<Events<T>>().add_system(
                Events::<T>::update_system
                    .in_base_set(CoreSet::First)
                    // in_schedule is the only difference with bevy's add_event
                    .in_schedule(PhysicsSchedule),
            );
        }
        self
    }

    fn add_event_channel<T: Event>(&mut self, receiver: Receiver<T>) -> &mut Self {
        assert!(
            !self.world.contains_resource::<ChannelReceiver<T>>(),
            "this event channel is already initialized",
        );

        self.add_event::<T>();
        self.add_system(
            channel_to_event::<T>
                .after(Events::<T>::update_system)
                .in_base_set(CoreSet::First),
        );
        self.insert_resource(ChannelReceiver(Mutex::new(receiver)));
        self
    }

    fn add_physics_event_channel<T: Event>(&mut self, receiver: Receiver<T>) -> &mut Self {
        assert!(
            !self.world.contains_resource::<ChannelReceiver<T>>(),
            "this event channel is already initialized",
        );

        self.add_physics_event::<T>();
        self.add_system(
            channel_to_event::<T>
                .after(Events::<T>::update_system)
                .in_base_set(CoreSet::First)
                .in_schedule(PhysicsSchedule),
        );
        self.insert_resource(ChannelReceiver(Mutex::new(receiver)));
        self
    }
}

fn channel_to_event<T: 'static + Send + Sync>(
    receiver: Res<ChannelReceiver<T>>,
    mut writer: EventWriter<T>,
) {
    // this should be the only system working with the receiver,
    // thus we always expect to get this lock
    let events = receiver.lock().expect("unable to acquire mutex lock");

    writer.send_batch(events.try_iter());
}