use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use physx::convex_mesh::ConvexMesh;
use physx::cooking::{TriangleMeshCookingResult, PxTriangleMeshDesc, ConvexMeshCookingResult, PxConvexMeshDesc};
use physx::prelude::*;
use physx::triangle_mesh::TriangleMesh;
use physx_sys::{PxConvexFlags, PxConvexFlag};
use std::ffi::c_void;
use crate::prelude::*;

use super::PxMaterial;
use super::resources::{BPxPhysics, BPxCooking};

#[derive(TypeUuid, Deref, DerefMut)]
#[uuid = "5351ec05-c0fd-426a-b35e-62008a6b10e1"]
pub struct BPxMaterial(Owner<PxMaterial>);

impl BPxMaterial {
    pub fn new(physics: &mut BPxPhysics, static_friction: f32, dynamic_friction: f32, restitution: f32) -> Self {
        physics.create_material(static_friction, dynamic_friction, restitution, ()).unwrap().into()
    }
}

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
    ConvexMesh(Owner<ConvexMesh>),
    TriangleMesh(Owner<TriangleMesh>),

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

impl From<Owner<ConvexMesh>> for BPxGeometry {
    fn from(value: Owner<ConvexMesh>) -> Self {
        Self::ConvexMesh(value)
    }
}

impl From<Owner<TriangleMesh>> for BPxGeometry {
    fn from(value: Owner<TriangleMesh>) -> Self {
        Self::TriangleMesh(value)
    }
}

impl BPxGeometry {
    pub fn ball(radius: f32) -> Self {
        Self::Sphere(PxSphereGeometry::new(radius))
    }

    pub fn halfspace() -> Self {
        Self::Plane(PxPlaneGeometry::new())
    }

    pub fn capsule(half_height: f32, radius: f32) -> Self {
        Self::Capsule(PxCapsuleGeometry::new(radius, half_height))
    }

    pub fn cuboid(hx: f32, hy: f32, hz: f32) -> Self {
        Self::Box(PxBoxGeometry::new(hx / 2., hy / 2., hz / 2.))
    }

    pub fn convex_mesh(physics: &mut BPxPhysics, cooking: &BPxCooking, verts: &[Vec3]) -> Self {
        let verts = verts.iter().map(|v| v.to_physx()).collect::<Vec<_>>();

        let mut mesh_desc = PxConvexMeshDesc::new();
        mesh_desc.obj.points.count = verts.len() as u32;
        mesh_desc.obj.points.stride = std::mem::size_of::<PxVec3>() as u32;
        mesh_desc.obj.points.data = verts.as_ptr() as *const c_void;
        mesh_desc.obj.flags = PxConvexFlags { mBits: PxConvexFlag::eCOMPUTE_CONVEX as u16 };

        let mesh = match cooking.create_convex_mesh(physics.physics_mut(), &mesh_desc) {
            ConvexMeshCookingResult::Success(mesh) => mesh,
            ConvexMeshCookingResult::Failure => panic!("create_convex_mesh failure"),
            ConvexMeshCookingResult::InvalidDescriptor => panic!("create_convex_mesh invalid descriptor"),
            ConvexMeshCookingResult::PolygonsLimitReached => panic!("create_convex_mesh polygon limit reached"),
            ConvexMeshCookingResult::ZeroAreaTestFailed => panic!("create_convex_mesh zero area test failed"),
        };

        Self::ConvexMesh(mesh)
    }

    pub fn trimesh(physics: &mut BPxPhysics, cooking: &BPxCooking, verts: &[Vec3], indices: &[[u32; 3]]) -> Self {
        let verts = verts.iter().map(|v| v.to_physx()).collect::<Vec<_>>();

        let mut mesh_desc = PxTriangleMeshDesc::new();
        mesh_desc.obj.points.count = verts.len() as u32;
        mesh_desc.obj.points.stride = std::mem::size_of::<PxVec3>() as u32;
        mesh_desc.obj.points.data = verts.as_ptr() as *const c_void;

        mesh_desc.obj.triangles.count = indices.len() as u32;
        mesh_desc.obj.triangles.stride = std::mem::size_of::<[u32; 3]>() as u32;
        mesh_desc.obj.triangles.data = indices.as_ptr() as *const c_void;

        let mesh = match cooking.create_triangle_mesh(physics.physics_mut(), &mesh_desc) {
            TriangleMeshCookingResult::Success(mesh) => mesh,
            TriangleMeshCookingResult::Failure => panic!("create_triangle_mesh failure"),
            TriangleMeshCookingResult::InvalidDescriptor => panic!("create_triangle_mesh invalid descriptor"),
            TriangleMeshCookingResult::LargeTriangle => panic!("create_triangle_mesh large triangle"),
        };

        Self::TriangleMesh(mesh)
    }

    pub fn cylinder(physics: &mut BPxPhysics, cooking: &BPxCooking, half_height: f32, radius: f32, segments: usize) -> Self {
        let mut points = vec![Vec3::default(); 2 * segments];

        for i in 0..segments {
            let cos_theta = (i as f32 * std::f32::consts::PI * 2. / segments as f32).cos();
            let sin_theta = (i as f32 * std::f32::consts::PI * 2. / segments as f32).sin();
            let y = radius * cos_theta;
            let z = radius * sin_theta;
            points[2 * i + 0] = Vec3::new(-half_height, y, z);
            points[2 * i + 1] = Vec3::new(half_height, y, z);
        }

        Self::convex_mesh(physics, cooking, &points)
    }
}
