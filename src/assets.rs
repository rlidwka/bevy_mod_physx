use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use physx::convex_mesh::ConvexMesh;
use physx::prelude::*;
use physx::triangle_mesh::TriangleMesh;
use super::PxMaterial;

#[derive(TypeUuid, Deref, DerefMut)]
#[uuid = "5351ec05-c0fd-426a-b35e-62008a6b10e1"]
pub struct BPxMaterial(Owner<PxMaterial>);

impl From<Owner<PxMaterial>> for BPxMaterial {
    fn from(value: Owner<PxMaterial>) -> Self {
        BPxMaterial(value)
    }
}

#[derive(TypeUuid)]
#[uuid = "db246120-e6af-4ebf-a95a-a6efe1c54d9f"]
pub enum BPxGeometry {
    Sphere(PxSphereGeometry),
    Plane(PxPlaneGeometry),
    Capsule(PxCapsuleGeometry),
    Box(PxBoxGeometry),

    // for convexmesh and triangle mesh we have to own the mesh,
    // so it's simpler to construct geometry on demand
    ConvexMesh(ConvexMesh),
    TriangleMesh(TriangleMesh),

    // TODO: height fields not implemented
    //HeightField(PxHeightFieldGeometry),
}

impl From<PxSphereGeometry> for BPxGeometry {
    fn from(value: PxSphereGeometry) -> Self {
        Self::Sphere(value)
    }
}

impl From<PxPlaneGeometry> for BPxGeometry {
    fn from(value: PxPlaneGeometry) -> Self {
        Self::Plane(value)
    }
}

impl From<PxCapsuleGeometry> for BPxGeometry {
    fn from(value: PxCapsuleGeometry) -> Self {
        Self::Capsule(value)
    }
}

impl From<PxBoxGeometry> for BPxGeometry {
    fn from(value: PxBoxGeometry) -> Self {
        Self::Box(value)
    }
}

impl From<ConvexMesh> for BPxGeometry {
    fn from(value: ConvexMesh) -> Self {
        Self::ConvexMesh(value)
    }
}

impl From<TriangleMesh> for BPxGeometry {
    fn from(value: TriangleMesh) -> Self {
        Self::TriangleMesh(value)
    }
}
