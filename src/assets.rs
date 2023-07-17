use std::ffi::c_void;
use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use bevy::reflect::{TypePath, TypeUuid};
use physx::convex_mesh::ConvexMesh;
use physx::cooking::{
    ConvexMeshCookingResult,
    PxConvexMeshDesc,
    PxCooking,
    PxCookingParams,
    PxHeightFieldDesc,
    PxTriangleMeshDesc,
    TriangleMeshCookingResult,
};
use physx::prelude::*;
use physx::triangle_mesh::TriangleMesh;
use physx_sys::{
    PxBitAndByte,
    PxConvexFlag,
    PxConvexFlags,
    PxConvexMeshGeometryFlag,
    PxConvexMeshGeometryFlags,
    PxHeightFieldFormat,
    PxHeightFieldSample,
    PxMeshGeometryFlag,
    PxMeshGeometryFlags,
};

use crate::prelude::{self as bpx, *};
use crate::types::PxMaterial;

#[derive(TypeUuid, TypePath, Deref, DerefMut)]
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

#[derive(TypeUuid, TypePath, Clone, Deref, DerefMut)]
#[uuid = "db246120-e6af-4ebf-a95a-a6efe1c54d9f"]
pub struct Geometry {
    pub obj: GeometryInner,
}

#[derive(Clone)]
pub enum GeometryInner {
    Sphere(PxSphereGeometry),
    Capsule(PxCapsuleGeometry),
    Box(PxBoxGeometry),

    // physx plane always faces in X direction,
    // but it's convenient to be able to customize this (similar to rapier api),
    // thus a lot of complications here
    Plane {
        plane: PxPlaneGeometry,
        normal: Vec3,
    },

    // for convexmesh and triangle mesh we have to own the mesh,
    // so it's simpler to construct geometry on demand
    ConvexMesh {
        mesh: Arc<Mutex<Owner<ConvexMesh>>>,
        scale: Vec3,
        rotation: Quat,
        flags: PxConvexMeshGeometryFlags,
    },

    TriangleMesh {
        mesh: Arc<Mutex<Owner<TriangleMesh>>>,
        scale: Vec3,
        rotation: Quat,
        flags: PxMeshGeometryFlags,
    },

    HeightField {
        mesh: Arc<Mutex<Owner<HeightField>>>,
        scale: Vec3,
        flags: PxMeshGeometryFlags,
    },
}

impl From<PxSphereGeometry> for Geometry {
    fn from(value: PxSphereGeometry) -> Self {
        Self { obj: GeometryInner::Sphere(value) }
    }
}

impl From<PxPlaneGeometry> for Geometry {
    fn from(value: PxPlaneGeometry) -> Self {
        // makes more sense to default normal to Y axis (ground), but physx defaults to X axis
        Self { obj: GeometryInner::Plane { plane: value, normal: Vec3::X } }
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
        Self { obj: GeometryInner::ConvexMesh {
            mesh: Arc::new(Mutex::new(value)),
            scale: Vec3::ONE,
            rotation: Quat::IDENTITY,
            flags: PxConvexMeshGeometryFlags { mBits: 0 },
        } }
    }
}

impl From<Owner<TriangleMesh>> for Geometry {
    fn from(value: Owner<TriangleMesh>) -> Self {
        Self { obj: GeometryInner::TriangleMesh {
            mesh: Arc::new(Mutex::new(value)),
            scale: Vec3::ONE,
            rotation: Quat::IDENTITY,
            flags: PxMeshGeometryFlags { mBits: 0 },
        } }
    }
}

impl From<Owner<HeightField>> for Geometry {
    fn from(value: Owner<HeightField>) -> Self {
        Self { obj: GeometryInner::HeightField {
            mesh: Arc::new(Mutex::new(value)),
            scale: Vec3::ONE,
            flags: PxMeshGeometryFlags { mBits: 0 },
        } }
    }
}

impl Geometry {
    pub fn ball(radius: f32) -> Self {
        PxSphereGeometry::new(radius).into()
    }

    pub fn halfspace(outward_normal: Vec3) -> Self {
        let Some(outward_normal) = outward_normal.try_normalize() else {
            panic!("halfspace outward normal is zero");
        };
        Self { obj: GeometryInner::Plane { plane: PxPlaneGeometry::new(), normal: outward_normal } }
    }

    pub fn capsule(half_height: f32, radius: f32) -> Self {
        PxCapsuleGeometry::new(radius, half_height).into()
    }

    pub fn cuboid(half_x: f32, half_y: f32, half_z: f32) -> Self {
        PxBoxGeometry::new(half_x, half_y, half_z).into()
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

    pub fn with_scale(mut self, new_scale: Vec3) -> Self {
        match self.obj {
            GeometryInner::ConvexMesh { ref mut scale, .. } => { *scale = new_scale; }
            GeometryInner::TriangleMesh { ref mut scale, .. } => { *scale = new_scale; }
            GeometryInner::HeightField { ref mut scale, .. } => { *scale = new_scale; }
            _ => {
                bevy::log::warn!("unable to set scale, wrong geometry type (not ConvexMesh, TriangleMesh or HeightField)");
            }
        };
        self
    }

    pub fn with_rotation(mut self, new_rotation: Quat) -> Self {
        match &mut self.obj {
            GeometryInner::ConvexMesh { ref mut rotation, .. } => { *rotation = new_rotation; }
            GeometryInner::TriangleMesh { ref mut rotation, .. } => { *rotation = new_rotation; }
            _ => {
                bevy::log::warn!("unable to set rotation, wrong geometry type (not ConvexMesh or TriangleMesh)");
            }
        };
        self
    }

    pub fn with_tight_bounds(mut self, tight_bounds: bool) -> Self {
        match &mut self.obj {
            GeometryInner::ConvexMesh { ref mut flags, .. } => {
                *flags = if tight_bounds {
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
            GeometryInner::TriangleMesh { ref mut flags, .. } => {
                *flags = if double_sided {
                    PxMeshGeometryFlags { mBits: PxMeshGeometryFlag::eDOUBLE_SIDED as _ }
                } else {
                    PxMeshGeometryFlags { mBits: 0 }
                }
            }
            GeometryInner::HeightField { ref mut flags, .. } => {
                *flags = if double_sided {
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
pub struct GeometryInnerPlane {
    pub plane: PxPlaneGeometry,
    pub normal: Vec3,
}
