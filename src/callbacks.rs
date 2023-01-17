use physx::prelude::*;
use super::*;

pub struct OnCollision;
impl CollisionCallback for OnCollision {
    fn on_collision(&mut self, _header: &physx_sys::PxContactPairHeader, _pairs: &[physx_sys::PxContactPair]) {}
}

pub struct OnTrigger;
impl TriggerCallback for OnTrigger {
    fn on_trigger(&mut self, _pairs: &[physx_sys::PxTriggerPair]) {}
}

pub struct OnConstraintBreak;
impl ConstraintBreakCallback for OnConstraintBreak {
    fn on_constraint_break(&mut self, _constraints: &[physx_sys::PxConstraintInfo]) {}
}

pub struct OnWakeSleep;
impl WakeSleepCallback<PxArticulationLink, PxRigidStatic, PxRigidDynamic> for OnWakeSleep {
    fn on_wake_sleep(&mut self, _actors: &[&physx::actor::ActorMap<PxArticulationLink, PxRigidStatic, PxRigidDynamic>], _is_waking: bool) {}
}

pub struct OnAdvance;
impl AdvanceCallback<PxArticulationLink, PxRigidDynamic> for OnAdvance {
    fn on_advance(&self, _actors: &[&physx::rigid_body::RigidBodyMap<PxArticulationLink, PxRigidDynamic>], _transforms: &[PxTransform]) {}
}
