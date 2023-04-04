use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use physx::convex_mesh::ConvexMesh;
use physx::cooking::{TriangleMeshCookingResult, PxTriangleMeshDesc, ConvexMeshCookingResult, PxConvexMeshDesc, PxHeightFieldDesc, self};
use physx::prelude::*;
use physx::triangle_mesh::TriangleMesh;
use physx_sys::{PxConvexFlags, PxHeightFieldSample, PxHeightFieldFormat, PxConvexMeshGeometryFlags, PxMeshGeometryFlags};
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
    Plane(GeometryInnerPlane),
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
        // makes more sense to default normal to Y axis (ground), but physx defaults to X axis
        Self { obj: GeometryInner::Plane(GeometryInnerPlane { plane: value, normal: Vec3::X }) }
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
            flags: ConvexMeshGeometryFlags::TightBounds,
        }) }
    }
}

impl From<Owner<TriangleMesh>> for Geometry {
    fn from(value: Owner<TriangleMesh>) -> Self {
        Self { obj: GeometryInner::TriangleMesh(GeometryInnerTriangleMesh {
            mesh: Arc::new(Mutex::new(value)),
            scale: Vec3::ONE,
            rotation: Quat::IDENTITY,
            flags: MeshGeometryFlags::TightBounds,
        }) }
    }
}

impl From<Owner<HeightField>> for Geometry {
    fn from(value: Owner<HeightField>) -> Self {
        Self { obj: GeometryInner::HeightField(GeometryInnerHeightField {
            hfield: Arc::new(Mutex::new(value)),
            scale: Vec3::ONE,
            flags: MeshGeometryFlags::TightBounds,
        }) }
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
        Self { obj: GeometryInner::Plane(GeometryInnerPlane { plane: PxPlaneGeometry::new(), normal: outward_normal }) }
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
        mesh_desc.obj.flags = PxConvexFlags::ComputeConvex;

        let params = cooking::PxCookingParams::new(physics.physics()).unwrap();
        match cooking::create_convex_mesh(physics.physics_mut(), &params, &mesh_desc) {
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

        let params = cooking::PxCookingParams::new(physics.physics()).unwrap();
        match cooking::create_triangle_mesh(physics.physics_mut(), &params, &mesh_desc) {
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
            samples.push(HeightFieldSample::new(height, default(), default(), false));
        }

        let mut hfield_desc = PxHeightFieldDesc::new();
        hfield_desc.obj.format = PxHeightFieldFormat::S16Tm;
        hfield_desc.obj.nbColumns = num_cols as u32;
        hfield_desc.obj.nbRows = num_rows as u32;
        hfield_desc.obj.samples.stride = std::mem::size_of::<PxHeightFieldSample>() as u32;
        hfield_desc.obj.samples.data = samples.as_ptr() as *const c_void;

        let mesh = cooking::create_height_field(physics.physics_mut(), &hfield_desc)
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
                obj.flags.set(ConvexMeshGeometryFlags::TightBounds, tight_bounds);
            }
            GeometryInner::TriangleMesh(ref mut obj) => {
                obj.flags.set(MeshGeometryFlags::TightBounds, tight_bounds);
            }
            GeometryInner::HeightField(ref mut obj) => {
                obj.flags.set(MeshGeometryFlags::TightBounds, tight_bounds);
            }
            _ => {
                bevy::log::warn!("unable to set tight bounds, wrong geometry type (not ConvexMesh, TriangleMesh or HeightField)");
            }
        };
        self
    }

    pub fn with_double_sided(mut self, double_sided: bool) -> Self {
        match &mut self.obj {
            GeometryInner::TriangleMesh(ref mut obj) => {
                obj.flags.set(MeshGeometryFlags::DoubleSided, double_sided);
            }
            GeometryInner::HeightField(ref mut obj) => {
                obj.flags.set(MeshGeometryFlags::DoubleSided, double_sided);
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
