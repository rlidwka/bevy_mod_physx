use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use physx::prelude::*;
use super::PxMaterial;

#[derive(TypeUuid, Deref, DerefMut)]
#[uuid = "5351ec05-c0fd-426a-b35e-62008a6b10e1"]
pub struct BPxMaterial(Owner<PxMaterial>);

impl From<Owner<PxMaterial>> for BPxMaterial {
    fn from(value: Owner<PxMaterial>) -> Self {
        BPxMaterial(value)
    }
}

#[derive(TypeUuid, Deref, DerefMut)]
#[uuid = "db246120-e6af-4ebf-a95a-a6efe1c54d9f"]
pub struct BPxGeometry(Box<dyn Geometry + Send + Sync>);

impl<T> From<T> for BPxGeometry where T: Geometry + Send + Sync + 'static {
    fn from(value: T) -> Self {
        BPxGeometry(Box::new(value))
    }
}
