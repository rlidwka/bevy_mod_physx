use crate::components::ShapeHandle;
use crate::prelude::{Scene, *};
use bevy::prelude::*;
use physx::traits::Class;
use physx_sys::{PxShape_setContactOffset_mut, PxShape_setRestOffset_mut};

#[derive(Component, Debug, Default, PartialEq, Clone, Copy, Reflect, FromReflect)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[reflect(Component, Default)]
pub struct ShapeOffsets {
    pub contact_offset: f32,
    pub rest_offset: f32,
}

pub struct ShapeOffsetsPlugin;

impl Plugin for ShapeOffsetsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ShapeOffsets>();
        app.add_system(shape_offsets_sync.in_base_set(PhysicsSet::Sync).in_schedule(PhysicsSchedule));
    }
}

pub fn shape_offsets_sync(
    mut scene: ResMut<Scene>,
    mut actors: Query<(Option<&mut ShapeHandle>, &ShapeOffsets), Changed<ShapeOffsets>>
) {
    // this function only applies user defined properties,
    // there's nothing to get back from physx engine
    for (handle, offsets) in actors.iter_mut() {
        if let Some(mut handle) = handle {
            let mut handle = handle.get_mut(&mut scene);

            unsafe {
                PxShape_setContactOffset_mut(handle.as_mut_ptr(), offsets.contact_offset);
                PxShape_setRestOffset_mut(handle.as_mut_ptr(), offsets.rest_offset);
            };
        } else {
            bevy::log::warn!("ShapeOffsets component exists, but it's not a shape");
        };
    }
}
