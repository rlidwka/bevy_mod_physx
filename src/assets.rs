use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use physx::convex_mesh::ConvexMesh;
use physx::cooking::{TriangleMeshCookingResult, PxTriangleMeshDesc, ConvexMeshCookingResult, PxConvexMeshDesc, PxHeightFieldDesc, PxCookingParams, PxCooking};
use physx::prelude::*;
use physx::triangle_mesh::TriangleMesh;
use physx_sys::{
    PxConvexFlags, PxConvexFlag, PxHeightFieldSample, PxBitAndByte, PxHeightFieldFormat, PxConvexMeshGeometryFlags,
    PxMeshGeometryFlags, PxConvexMeshGeometryFlag, PxMeshGeometryFlag
};
use std::ffi::c_void;
use std::sync::{Arc, Mutex};
use crate::prelude as bpx;
use crate::prelude::*;
use super::PxMaterial;

#[derive(TypeUuid, Deref, DerefMut)]
#[uuid = "5351ec05-c0fd-426a-b35e-62008a6b10e1"]
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

#[derive(TypeUuid, Clone, Deref, DerefMut)]
#[uuid = "db246120-e6af-4ebf-a95a-a6efe1c54d9f"]
pub struct Geometry {
    pub obj: GeometryInner,
}

#[derive(Clone)]
pub enum GeometryInner {
    Sphere(PxSphereGeometry),
    Plane(PxPlaneGeometry),
    Capsule(PxCapsuleGeometry),
    Box(PxBoxGeometry),

    // for convexmesh and triangle mesh we have to own the mesh,
    // so it's simpler to construct geometry on demand
    ConvexMesh(GeometryInnerConvexMesh),
    TriangleMesh(GeometryInnerTriangleMesh),
    HeightField(GeometryInnerHeightField),
}

impl From<PxSphereGeometry> for Geometry {
    fn from(value: PxSphereGeometry) -> Self {
        Self { obj: GeometryInner::Sphere(value) }
    }
}

impl From<PxPlaneGeometry> for Geometry {
    fn from(value: PxPlaneGeometry) -> Self {
        Self { obj: GeometryInner::Plane(value) }
    }
}

impl From<PxCapsuleGeometry> for Geometry {
    fn from(value: PxCapsuleGeometry) -> Self {
        Self { obj: GeometryInner::Capsule(value) }
    }
}

impl From<PxBoxGeometry> for Geometry {
    fn from(value: PxBoxGeometry) -> Self {
        Self { obj: GeometryInner::Box(value) }
    }
}

impl From<Owner<ConvexMesh>> for Geometry {
    fn from(value: Owner<ConvexMesh>) -> Self {
        Self { obj: GeometryInner::ConvexMesh(GeometryInnerConvexMesh {
            mesh: Arc::new(Mutex::new(value)),
            scale: Vec3::ONE,
            rotation: Quat::IDENTITY,
            flags: PxConvexMeshGeometryFlags { mBits: 0 },
        }) }
    }
}

impl From<Owner<TriangleMesh>> for Geometry {
    fn from(value: Owner<TriangleMesh>) -> Self {
        Self { obj: GeometryInner::TriangleMesh(GeometryInnerTriangleMesh {
            mesh: Arc::new(Mutex::new(value)),
            scale: Vec3::ONE,
            rotation: Quat::IDENTITY,
            flags: PxMeshGeometryFlags { mBits: 0 },
        }) }
    }
}

impl From<Owner<HeightField>> for Geometry {
    fn from(value: Owner<HeightField>) -> Self {
        Self { obj: GeometryInner::HeightField(GeometryInnerHeightField {
            hfield: Arc::new(Mutex::new(value)),
            scale: Vec3::ONE,
            flags: PxMeshGeometryFlags { mBits: 0 },
        }) }
    }
}

impl Geometry {
    pub fn ball(radius: f32) -> Self {
        PxSphereGeometry::new(radius).into()
    }

    pub fn halfspace() -> Self {
        PxPlaneGeometry::new().into()
    }

    pub fn capsule(half_height: f32, radius: f32) -> Self {
        PxCapsuleGeometry::new(radius, half_height).into()
    }

    pub fn cuboid(hx: f32, hy: f32, hz: f32) -> Self {
        PxBoxGeometry::new(hx / 2., hy / 2., hz / 2.).into()
    }

    pub fn convex_mesh(
        physics: &mut bpx::Physics,
        verts: &[Vec3],
    ) -> Result<Self, ConvexMeshCookingError> {
        let verts = verts.iter().map(|v| v.to_physx()).collect::<Vec<_>>();

        let mut mesh_desc = PxConvexMeshDesc::new();
        mesh_desc.obj.points.count = verts.len() as u32;
        mesh_desc.obj.points.stride = std::mem::size_of::<PxVec3>() as u32;
        mesh_desc.obj.points.data = verts.as_ptr() as *const c_void;
        mesh_desc.obj.flags = PxConvexFlags { mBits: PxConvexFlag::eCOMPUTE_CONVEX as u16 };

        let params = &PxCookingParams::new(&**physics).expect("failed to create cooking params");
        let cooking = PxCooking::new(physics.foundation_mut(), params).expect("failed to create cooking");

        match cooking.create_convex_mesh(physics.physics_mut(), &mesh_desc) {
            ConvexMeshCookingResult::Success(mesh) => Ok(mesh.into()),
            ConvexMeshCookingResult::Failure => Err(ConvexMeshCookingError::Failure),
            ConvexMeshCookingResult::InvalidDescriptor => Err(ConvexMeshCookingError::InvalidDescriptor),
            ConvexMeshCookingResult::PolygonsLimitReached => Err(ConvexMeshCookingError::PolygonsLimitReached),
            ConvexMeshCookingResult::ZeroAreaTestFailed => Err(ConvexMeshCookingError::ZeroAreaTestFailed),
        }
    }

    pub fn trimesh(
        physics: &mut bpx::Physics,
        verts: &[Vec3],
        indices: &[[u32; 3]],
    ) -> Result<Self, TriangleMeshCookingError> {
        let verts = verts.iter().map(|v| v.to_physx()).collect::<Vec<_>>();

        let mut mesh_desc = PxTriangleMeshDesc::new();
        mesh_desc.obj.points.count = verts.len() as u32;
        mesh_desc.obj.points.stride = std::mem::size_of::<PxVec3>() as u32;
        mesh_desc.obj.points.data = verts.as_ptr() as *const c_void;

        mesh_desc.obj.triangles.count = indices.len() as u32;
        mesh_desc.obj.triangles.stride = std::mem::size_of::<[u32; 3]>() as u32;
        mesh_desc.obj.triangles.data = indices.as_ptr() as *const c_void;

        let params = &PxCookingParams::new(&**physics).expect("failed to create cooking params");
        let cooking = PxCooking::new(physics.foundation_mut(), params).expect("failed to create cooking");

        match cooking.create_triangle_mesh(physics.physics_mut(), &mesh_desc) {
            TriangleMeshCookingResult::Success(mesh) => Ok(mesh.into()),
            TriangleMeshCookingResult::Failure => Err(TriangleMeshCookingError::Failure),
            TriangleMeshCookingResult::InvalidDescriptor => Err(TriangleMeshCookingError::InvalidDescriptor),
            TriangleMeshCookingResult::LargeTriangle => Err(TriangleMeshCookingError::LargeTriangle),
        }
    }

    pub fn cylinder(
        physics: &mut bpx::Physics,
        half_height: f32,
        radius: f32,
        segments: usize,
    ) -> Result<Self, ConvexMeshCookingError> {
        let mut points = vec![Vec3::default(); 2 * segments];

        for i in 0..segments {
            let cos_theta = (i as f32 * std::f32::consts::PI * 2. / segments as f32).cos();
            let sin_theta = (i as f32 * std::f32::consts::PI * 2. / segments as f32).sin();
            let y = radius * cos_theta;
            let z = radius * sin_theta;
            points[2 * i]    = Vec3::new(-half_height, y, z);
            points[2 * i + 1] = Vec3::new(half_height, y, z);
        }

        Self::convex_mesh(physics, &points)
    }

    pub fn heightfield(
        physics: &mut bpx::Physics,
        heights: &[i16],
        num_rows: usize,
        num_cols: usize,
    ) -> Self {
        assert_eq!(heights.len(), num_rows * num_cols, "invalid number of heights provided");

        let mut samples = Vec::with_capacity(num_rows * num_cols);

        for height in heights.iter().copied() {
            samples.push(PxHeightFieldSample {
                height,
                materialIndex0: PxBitAndByte { mData: 0 },
                materialIndex1: PxBitAndByte { mData: 0 },
            });
        }

        let mut hfield_desc = PxHeightFieldDesc::new();
        hfield_desc.obj.format = PxHeightFieldFormat::eS16_TM;
        hfield_desc.obj.nbColumns = num_cols as u32;
        hfield_desc.obj.nbRows = num_rows as u32;
        hfield_desc.obj.samples.stride = std::mem::size_of::<PxHeightFieldSample>() as u32;
        hfield_desc.obj.samples.data = samples.as_ptr() as *const c_void;

        let params = &PxCookingParams::new(&**physics).expect("failed to create cooking params");
        let cooking = PxCooking::new(physics.foundation_mut(), params).expect("failed to create cooking");

        let mesh = cooking.create_height_field(physics.physics_mut(), &hfield_desc)
            .expect("create_height_field failure");

        mesh.into()
    }

    pub fn with_scale(mut self, scale: Vec3) -> Self {
        match &mut self.obj {
            GeometryInner::ConvexMesh(ref mut obj) => { obj.scale = scale; }
            GeometryInner::TriangleMesh(ref mut obj) => { obj.scale = scale; }
            GeometryInner::HeightField(ref mut obj) => { obj.scale = scale; }
            _ => {
                bevy::log::warn!("unable to set scale, wrong geometry type (not ConvexMesh, TriangleMesh or HeightField)");
            }
        };
        self
    }

    pub fn with_rotation(mut self, rotation: Quat) -> Self {
        match &mut self.obj {
            GeometryInner::ConvexMesh(ref mut obj) => { obj.rotation = rotation; }
            GeometryInner::TriangleMesh(ref mut obj) => { obj.rotation = rotation; }
            _ => {
                bevy::log::warn!("unable to set rotation, wrong geometry type (not ConvexMesh or TriangleMesh)");
            }
        };
        self
    }

    pub fn with_tight_bounds(mut self, tight_bounds: bool) -> Self {
        match &mut self.obj {
            GeometryInner::ConvexMesh(ref mut obj) => {
                obj.flags = if tight_bounds {
                    PxConvexMeshGeometryFlags { mBits: PxConvexMeshGeometryFlag::eTIGHT_BOUNDS as _ }
                } else {
                    PxConvexMeshGeometryFlags { mBits: 0 }
                }
            }
            _ => {
                bevy::log::warn!("unable to set tight bounds, wrong geometry type (not ConvexMesh)");
            }
        };
        self
    }

    pub fn with_double_sided(mut self, double_sided: bool) -> Self {
        match &mut self.obj {
            GeometryInner::TriangleMesh(ref mut obj) => {
                obj.flags = if double_sided {
                    PxMeshGeometryFlags { mBits: PxMeshGeometryFlag::eDOUBLE_SIDED as _ }
                } else {
                    PxMeshGeometryFlags { mBits: 0 }
                }
            }
            GeometryInner::HeightField(ref mut obj) => {
                obj.flags = if double_sided {
                    PxMeshGeometryFlags { mBits: PxMeshGeometryFlag::eDOUBLE_SIDED as _ }
                } else {
                    PxMeshGeometryFlags { mBits: 0 }
                }
            }
            _ => {
                bevy::log::warn!("unable to set double sided, wrong geometry type (not TriangleMesh or HeightField)");
            }
        };
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConvexMeshCookingError {
    Failure,
    InvalidDescriptor,
    PolygonsLimitReached,
    ZeroAreaTestFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriangleMeshCookingError {
    Failure,
    InvalidDescriptor,
    LargeTriangle,
}

#[derive(Clone)]
pub struct GeometryInnerConvexMesh {
    pub mesh: Arc<Mutex<Owner<ConvexMesh>>>,
    pub scale: Vec3,
    pub rotation: Quat,
    pub flags: PxConvexMeshGeometryFlags,
}

#[derive(Clone)]
pub struct GeometryInnerTriangleMesh {
    pub mesh: Arc<Mutex<Owner<TriangleMesh>>>,
    pub scale: Vec3,
    pub rotation: Quat,
    pub flags: PxMeshGeometryFlags,
}

#[derive(Clone)]
pub struct GeometryInnerHeightField {
    pub hfield: Arc<Mutex<Owner<HeightField>>>,
    pub scale: Vec3,
    pub flags: PxMeshGeometryFlags,
}
