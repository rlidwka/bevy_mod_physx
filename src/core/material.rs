//! Material object represents a set of surface properties.
//!
//! This specifies static friction, dynamic friction and restitution of any surface.
use bevy::prelude::*;
use bevy::reflect::{TypePath, TypeUuid};
use physx::prelude::*;

use crate::prelude as bpx;
use crate::types::PxMaterial;

#[derive(TypeUuid, TypePath, Deref, DerefMut)]
#[uuid = "5351ec05-c0fd-426a-b35e-62008a6b10e1"]
/// Material object represents a set of surface properties.
pub struct Material(Owner<PxMaterial>);

impl Material {
    pub fn new(physics: &mut bpx::Physics, static_friction: f32, dynamic_friction: f32, restitution: f32) -> Self {
        physics.create_material(static_friction, dynamic_friction, restitution, ()).unwrap().into()
    }
}

impl From<Owner<PxMaterial>> for Material {
    fn from(value: Owner<PxMaterial>) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DefaultMaterial {
    pub static_friction: f32,
    pub dynamic_friction: f32,
    pub restitution: f32,
}

#[derive(Resource, Deref, DerefMut, Default)]
pub struct DefaultMaterialHandle(pub Handle<bpx::Material>);
