use bevy::prelude::*;
use physx::traits::Class;
use physx_sys::{
    PxFilterData,
    PxFilterData_new_2,
    PxShape_setQueryFilterData_mut,
    PxShape_setSimulationFilterData_mut,
};

use crate::components::ShapeHandle;
use crate::prelude::{Scene, *};

#[derive(Component, Debug, Default, PartialEq, Eq, Clone, Copy, Reflect)]
#[cfg_attr(feature = "serialize", derive(serde::Serialize, serde::Deserialize))]
#[reflect(Component, Default)]
pub struct ShapeFilterData {
    pub query_filter_data: [ u32; 4 ],
    pub simulation_filter_data: [ u32; 4 ],
}

pub struct ShapeFilterDataPlugin;

impl Plugin for ShapeFilterDataPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<ShapeFilterData>();
        app.add_systems(PhysicsSchedule, shape_filter_data.in_set(PhysicsSet::Sync));
    }
}

pub fn shape_filter_data(
    mut scene: ResMut<Scene>,
    mut actors: Query<
        (Option<&mut ShapeHandle>, Ref<ShapeFilterData>),
        Or<(Added<ShapeHandle>, Changed<ShapeFilterData>)>,
    >,
) {
    // this function only applies user defined properties,
    // there's nothing to get back from physx engine
    for (handle, filters) in actors.iter_mut() {
        if let Some(mut handle) = handle {
            let mut handle = handle.get_mut(&mut scene);

            unsafe {
                let [ word0, word1, word2, word3 ] = filters.query_filter_data;
                let pxfilterdata : PxFilterData = PxFilterData_new_2(word0, word1, word2, word3);
                PxShape_setQueryFilterData_mut(handle.as_mut_ptr(), &pxfilterdata as *const _);

                let [ word0, word1, word2, word3 ] = filters.simulation_filter_data;
                let pxfilterdata : PxFilterData = PxFilterData_new_2(word0, word1, word2, word3);
                PxShape_setSimulationFilterData_mut(handle.as_mut_ptr(), &pxfilterdata as *const _);
            };
        } else if !filters.is_added() {
            bevy::log::warn!("ShapeFilterData component exists, but it's not a shape");
        };
    }
}
