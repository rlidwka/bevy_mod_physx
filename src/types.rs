//! Monomorphized PhysX types used by Bevy plugin.
//!
use std::cell::RefCell;

use bevy::prelude::Entity;
use physx::prelude::*;

pub type PxMaterial = physx::material::PxMaterial<()>;
pub type PxShape = physx::shape::PxShape<Entity, PxMaterial>;
pub type PxArticulationLink = physx::articulation_link::PxArticulationLink<Entity, PxShape>;
pub type PxRigidStatic = physx::rigid_static::PxRigidStatic<Entity, PxShape>;
pub type PxRigidDynamic = physx::rigid_dynamic::PxRigidDynamic<Entity, PxShape>;
pub type PxArticulationReducedCoordinate =
    physx::articulation_reduced_coordinate::PxArticulationReducedCoordinate<Entity, PxArticulationLink>;

pub type PxScene = physx::scene::PxScene<
    (),
    PxArticulationLink,
    PxRigidStatic,
    PxRigidDynamic,
    PxArticulationReducedCoordinate,
    OnCollision,
    OnTrigger,
    OnConstraintBreak,
    OnWakeSleep,
    OnAdvance,
>;

/// This is called when certain contact events occur.
///
/// The method will be called for a pair of actors if one of the colliding
/// shape pairs requested contact notification. You request which events
/// are reported using the filter shader/callback mechanism.
///
/// Do not keep references to the passed objects, as they will be invalid
/// after this function returns.
pub struct OnCollision {
    callback: RefCell<Option<Box<dyn FnMut(&physx_sys::PxContactPairHeader, &[physx_sys::PxContactPair])>>>,
    initialized: bool,
}

unsafe impl Send for OnCollision {}
unsafe impl Sync for OnCollision {}

impl OnCollision {
    pub fn new(on_collision: impl FnMut(&physx_sys::PxContactPairHeader, &[physx_sys::PxContactPair]) + 'static) -> Self {
        Self {
            callback: RefCell::new(Some(Box::new(on_collision))),
            initialized: false,
        }
    }

    pub(crate) fn initialize(&self) -> Self {
        assert!(!self.initialized);
        Self {
            callback: RefCell::new(self.callback.borrow_mut().take()),
            initialized: true,
        }
    }
}

impl CollisionCallback for OnCollision {
    fn on_collision(&mut self, header: &physx_sys::PxContactPairHeader, pairs: &[physx_sys::PxContactPair]) {
        (self.callback.borrow_mut().as_mut().unwrap())(header, pairs);
    }
}

/// This is called with the current trigger pair events.
///
/// Shapes which have been marked as triggers using [ShapeFlag::TriggerShape]
/// will send events according to the pair flag specification in the filter shader.
pub struct OnTrigger {
    callback: RefCell<Option<Box<dyn FnMut(&[physx_sys::PxTriggerPair])>>>,
    initialized: bool,
}

unsafe impl Send for OnTrigger {}
unsafe impl Sync for OnTrigger {}

impl OnTrigger {
    pub fn new(on_trigger: impl FnMut(&[physx_sys::PxTriggerPair]) + 'static) -> Self {
        Self {
            callback: RefCell::new(Some(Box::new(on_trigger))),
            initialized: false,
        }
    }

    pub(crate) fn initialize(&self) -> Self {
        assert!(!self.initialized);
        Self {
            callback: RefCell::new(self.callback.borrow_mut().take()),
            initialized: true,
        }
    }
}

impl TriggerCallback for OnTrigger {
    fn on_trigger(&mut self, pairs: &[physx_sys::PxTriggerPair]) {
        (self.callback.borrow_mut().as_mut().unwrap())(pairs);
    }
}

/// This is called when a breakable constraint breaks.
pub struct OnConstraintBreak {
    callback: RefCell<Option<Box<dyn FnMut(&[physx_sys::PxConstraintInfo])>>>,
    initialized: bool,
}

unsafe impl Send for OnConstraintBreak {}
unsafe impl Sync for OnConstraintBreak {}

impl OnConstraintBreak {
    pub fn new(on_constraint_break: impl FnMut(&[physx_sys::PxConstraintInfo]) + 'static) -> Self {
        Self {
            callback: RefCell::new(Some(Box::new(on_constraint_break))),
            initialized: false,
        }
    }

    pub(crate) fn initialize(&self) -> Self {
        assert!(!self.initialized);
        Self {
            callback: RefCell::new(self.callback.borrow_mut().take()),
            initialized: true,
        }
    }
}

impl ConstraintBreakCallback for OnConstraintBreak {
    fn on_constraint_break(&mut self, constraints: &[physx_sys::PxConstraintInfo]) {
        (self.callback.borrow_mut().as_mut().unwrap())(constraints);
    }
}

/// This is called with the actors which have just been woken up or put to sleep.
pub struct OnWakeSleep {
    callback: RefCell<Option<Box<dyn FnMut(&[&physx::actor::ActorMap<PxArticulationLink, PxRigidStatic, PxRigidDynamic>], bool)>>>,
    initialized: bool,
}

unsafe impl Send for OnWakeSleep {}
unsafe impl Sync for OnWakeSleep {}

impl OnWakeSleep {
    pub fn new(on_wake_sleep: impl FnMut(&[&physx::actor::ActorMap<PxArticulationLink, PxRigidStatic, PxRigidDynamic>], bool) + 'static) -> Self {
        Self {
            callback: RefCell::new(Some(Box::new(on_wake_sleep))),
            initialized: false,
        }
    }

    pub(crate) fn initialize(&self) -> Self {
        assert!(!self.initialized);
        Self {
            callback: RefCell::new(self.callback.borrow_mut().take()),
            initialized: true,
        }
    }
}

impl WakeSleepCallback<PxArticulationLink, PxRigidStatic, PxRigidDynamic> for OnWakeSleep {
    fn on_wake_sleep(&mut self, actors: &[&physx::actor::ActorMap<PxArticulationLink, PxRigidStatic, PxRigidDynamic>], is_waking: bool) {
        (self.callback.borrow_mut().as_mut().unwrap())(actors, is_waking);
    }
}

/// Provides early access to the new pose of moving rigid bodies.
///
/// When this call occurs, rigid bodies having the [RigidBodyFlag::EnablePoseIntegrationPreview]
/// flag set, were moved by the simulation and their new poses can be accessed
/// through the provided buffers.
pub struct OnAdvance {
    callback: RefCell<Option<Box<dyn FnMut(&[&physx::rigid_body::RigidBodyMap<PxArticulationLink, PxRigidDynamic>], &[PxTransform])>>>,
    initialized: bool,
}

unsafe impl Send for OnAdvance {}
unsafe impl Sync for OnAdvance {}

impl OnAdvance {
    pub fn new(on_advance: impl FnMut(&[&physx::rigid_body::RigidBodyMap<PxArticulationLink, PxRigidDynamic>], &[PxTransform]) + 'static) -> Self {
        Self {
            callback: RefCell::new(Some(Box::new(on_advance))),
            initialized: false,
        }
    }

    pub(crate) fn initialize(&self) -> Self {
        assert!(!self.initialized);
        Self {
            callback: RefCell::new(self.callback.borrow_mut().take()),
            initialized: true,
        }
    }
}

impl AdvanceCallback<PxArticulationLink, PxRigidDynamic> for OnAdvance {
    fn on_advance(&self, actors: &[&physx::rigid_body::RigidBodyMap<PxArticulationLink, PxRigidDynamic>], transforms: &[PxTransform]) {
        (self.callback.borrow_mut().as_mut().unwrap())(actors, transforms);
    }
}
