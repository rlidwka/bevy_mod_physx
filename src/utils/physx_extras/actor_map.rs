use physx::actor::{ActorMap, ActorType};
use physx::prelude::*;

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
