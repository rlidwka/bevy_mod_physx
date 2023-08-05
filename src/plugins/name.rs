//! Assigns a name for the object to be displayed in PVD.
//!
//! This is for debugging and is not used by the SDK.
//! It is displayed for example in PhysX Visual Debugger.
//!
//! In order to use this feature, you should insert [NameFormatter]
//! resource:
//! ```no_run
//! app.insert_resource(NameFormatter(|entity, name| {
//!     let str = if let Some(name) = name {
//!         format!("{name} ({entity:?})")
//!     } else {
//!         format!("({entity:?})")
//!     };
//!
//!     std::borrow::Cow::Owned(CString::new(str).unwrap())
//! }));
//! ```
use std::borrow::Cow;
use std::ffi::CString;

use bevy::prelude::*;
use physx::traits::Class;
use physx_sys::{
    PxActor_setName_mut,
    PxArticulationReducedCoordinate_setName_mut,
    PxShape_setName_mut,
};

use crate::components::{
    ArticulationLinkHandle,
    ArticulationRootHandle,
    RigidDynamicHandle,
    RigidStaticHandle,
    ShapeHandle,
};
use crate::prelude::{Scene, *};

#[derive(Resource)]
pub struct NameFormatter(pub fn(Entity, Option<&Name>) -> Cow<'static, CString>);

#[derive(Component)]
struct PxName(Cow<'static, CString>);

pub struct NamePlugin;

impl Plugin for NamePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PhysicsSchedule,
            name_sync
                .after(crate::systems::scene_simulate)
                .run_if(resource_exists::<NameFormatter>())
                .in_set(PhysicsSet::Simulate),
        );
    }
}

fn name_sync(
    mut commands: Commands,
    mut scene: ResMut<Scene>,
    mut handles: Query<
        (
            Entity,
            Option<&Name>,
            Option<&PxName>,
            Option<&mut ArticulationLinkHandle>,
            Option<&mut ArticulationRootHandle>,
            Option<&mut RigidDynamicHandle>,
            Option<&mut RigidStaticHandle>,
            Option<&mut ShapeHandle>,
        ),
        Or<(
            Added<ArticulationLinkHandle>,
            Added<ArticulationRootHandle>,
            Added<RigidDynamicHandle>,
            Added<RigidStaticHandle>,
            Added<ShapeHandle>,
            // changing name doesn't affect PVD,
            // as only first name sent to it is displayed
            //Changed<Name>,
        )>,
    >,
    fmt: Res<NameFormatter>,
) {
    for (entity, bevy_name, existing_name, hlink, hroot, hdyn, hstat, hshape) in handles.iter_mut() {
        if hlink.is_none() && hroot.is_none() && hdyn.is_none() && hstat.is_none() && hshape.is_none() {
            continue;
        }

        let name = existing_name.map(|x| x.0.as_ptr()).unwrap_or_else(|| {
            let name = fmt.0(entity, bevy_name);
            let ptr = name.as_ptr();
            // SAFETY: PxName is stored in the entity for as long as entity lives,
            // so pointer will always be valid
            commands.entity(entity).insert(PxName(name));
            ptr
        });

        if let Some(mut h) = hroot {
            // this doesn't show up in physx debugger for unknown reasons
            // (works if we set it straight away when articulation is created)
            let mut h = h.get_mut(&mut scene);
            unsafe { PxArticulationReducedCoordinate_setName_mut(h.as_mut_ptr(), name) };
        }

        if let Some(mut h) = hlink {
            let mut h = h.get_mut(&mut scene);
            unsafe { PxActor_setName_mut(h.as_mut_ptr(), name) };
        }

        if let Some(mut h) = hdyn {
            let mut h = h.get_mut(&mut scene);
            unsafe { PxActor_setName_mut(h.as_mut_ptr(), name) };
        }

        if let Some(mut h) = hstat {
            let mut h = h.get_mut(&mut scene);
            unsafe { PxActor_setName_mut(h.as_mut_ptr(), name) };
        }

        if let Some(mut h) = hshape {
            let mut h = h.get_mut(&mut scene);
            unsafe { PxShape_setName_mut(h.as_mut_ptr(), name) };
        }
    }
}
