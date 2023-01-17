use bevy::prelude::*;
use physx::prelude::*;
use super::{assets::{BPxGeometry, BPxMaterial}, PxRigidDynamic, PxRigidStatic};

pub struct BPxActorDynamic {
    pub geometry: Handle<BPxGeometry>,
    pub material: Handle<BPxMaterial>,
    pub density: f32,
    pub shape_offset: Transform,
}

impl Default for BPxActorDynamic {
    fn default() -> Self {
        Self {
            geometry: Default::default(),
            material: Default::default(),
            density: 1.0,
            shape_offset: Transform::IDENTITY,
        }
    }
}

pub struct BPxActorStatic {
    pub geometry: Handle<BPxGeometry>,
    pub material: Handle<BPxMaterial>,
    pub shape_offset: Transform,
}

impl Default for BPxActorStatic {
    fn default() -> Self {
        Self {
            geometry: Default::default(),
            material: Default::default(),
            shape_offset: Transform::IDENTITY,
        }
    }
}

pub struct BPxActorPlane {
    pub normal: Vec3,
    pub offset: f32,
    pub material: Handle<BPxMaterial>,
}

impl Default for BPxActorPlane {
    fn default() -> Self {
        Self {
            normal: Vec3::Y,
            offset: 0.,
            material: Default::default(),
        }
    }
}

#[derive(Component)]
pub enum BPxActor {
    Dynamic {
        geometry: Handle<BPxGeometry>,
        material: Handle<BPxMaterial>,
        density: f32,
        shape_offset: Transform,
    },
    Static {
        geometry: Handle<BPxGeometry>,
        material: Handle<BPxMaterial>,
        shape_offset: Transform,
    },
    Plane {
        normal: Vec3,
        offset: f32,
        material: Handle<BPxMaterial>,
    },
}

#[derive(Component, Deref, DerefMut)]
pub struct BPxRigidDynamic(Owner<PxRigidDynamic>);

impl BPxRigidDynamic {
    pub fn new(px_rigid_dynamic: Owner<PxRigidDynamic>) -> Self {
        Self(px_rigid_dynamic)
    }
}

#[derive(Component, Deref, DerefMut)]
pub struct BPxRigidStatic(Owner<PxRigidStatic>);

impl BPxRigidStatic {
    pub fn new(px_rigid_static: Owner<PxRigidStatic>) -> Self {
        Self(px_rigid_static)
    }
}

#[derive(Component, Debug, Default, PartialEq, Reflect, Clone, Copy)]
pub struct BPxVelocity {
    pub linvel: Vec3,
    pub angvel: Vec3,
}

impl BPxVelocity {
    pub fn new(linvel: Vec3, angvel: Vec3) -> Self {
        Self { linvel, angvel }
    }

    pub fn zero() -> Self {
        Self { ..default() }
    }

    pub fn linear(linvel: Vec3) -> Self {
        Self { linvel, ..default() }
    }

    pub fn angular(angvel: Vec3) -> Self {
        Self { angvel, ..default() }
    }
}
