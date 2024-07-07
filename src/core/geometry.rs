//! Geometry object defines the characteristics of a spatial object.
//!
//! Following geometries are supported:
//!  - basic shapes
//!    - sphere
//!    - capsule
//!    - box
//!    - plane
//!  - cooked shapes
//!    - convex mesh
//!    - triangle mesh
//!    - heightfield
//!  - bevy meshes
//!    - convex mesh
//!    - triangle mesh
//!
use std::ffi::c_void;
use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use bevy::render::mesh::{Indices, VertexAttributeValues};
use physx::convex_mesh::ConvexMesh;
use physx::cooking::{
    self,
    ConvexMeshCookingResult,
    PxConvexMeshDesc,
    PxHeightFieldDesc,
    PxTriangleMeshDesc,
    TriangleMeshCookingResult,
};
use physx::prelude::*;
use physx::traits::Class;
use physx::triangle_mesh::TriangleMesh;
use physx_sys::{
    PxConvexFlags,
    PxConvexMeshGeometryFlags,
    PxHeightFieldFormat,
    PxHeightFieldSample,
    PxMeshGeometryFlags,
    PxMeshScale,
    PxMeshScale_new,
    PxMeshScale_new_3,
};

use crate::prelude::{self as bpx, *};

#[derive(Asset, TypePath, Clone, Deref, DerefMut)]
/// Geometry object defines the characteristics of a spatial object.
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
        normal: Direction3d,
    },

    // for convexmesh and triangle mesh we have to own the mesh,
    // so it's simpler to construct geometry on demand
    ConvexMesh {
        mesh: Arc<Mutex<Owner<ConvexMesh>>>,
        scale: PxMeshScale,
        flags: PxConvexMeshGeometryFlags,
    },

    TriangleMesh {
        mesh: Arc<Mutex<Owner<TriangleMesh>>>,
        scale: PxMeshScale,
        flags: PxMeshGeometryFlags,
    },

    HeightField {
        mesh: Arc<Mutex<Owner<HeightField>>>,
        scale: PxMeshScale,
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
        Self { obj: GeometryInner::Plane { plane: value, normal: Direction3d::X } }
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
            scale: unsafe { PxMeshScale_new() },
            flags: ConvexMeshGeometryFlags::TightBounds,
        } }
    }
}

impl From<Owner<TriangleMesh>> for Geometry {
    fn from(value: Owner<TriangleMesh>) -> Self {
        Self { obj: GeometryInner::TriangleMesh {
            mesh: Arc::new(Mutex::new(value)),
            scale: unsafe { PxMeshScale_new() },
            flags: MeshGeometryFlags::TightBounds,
        } }
    }
}

impl From<Owner<HeightField>> for Geometry {
    fn from(value: Owner<HeightField>) -> Self {
        Self { obj: GeometryInner::HeightField {
            mesh: Arc::new(Mutex::new(value)),
            scale: unsafe { PxMeshScale_new() },
            flags: MeshGeometryFlags::TightBounds,
        } }
    }
}

impl From<Sphere> for Geometry {
    fn from(sphere: Sphere) -> Self {
        PxSphereGeometry::new(sphere.radius).into()
    }
}

impl From<Plane3d> for Geometry {
    fn from(plane: Plane3d) -> Self {
        Self {
            obj: GeometryInner::Plane {
                plane: PxPlaneGeometry::new(),
                normal: plane.normal,
            },
        }
    }
}

impl From<Capsule3d> for Geometry {
    fn from(capsule: Capsule3d) -> Self {
        PxCapsuleGeometry::new(capsule.radius, capsule.half_length).into()
    }
}

impl From<Cuboid> for Geometry {
    fn from(cuboid: Cuboid) -> Self {
        PxBoxGeometry::new(cuboid.half_size.x, cuboid.half_size.y, cuboid.half_size.z).into()
    }
}

impl Geometry {
    #[deprecated(
        since = "0.5.0",
        note = "please use Bevy's `Sphere` primitive (e.g. using `Geometry::from`)"
    )]
    pub fn ball(radius: f32) -> Self {
        Sphere::new(radius).into()
    }

    #[deprecated(
        since = "0.5.0",
        note = "please use Bevy's `Plane3d` primitive (e.g. using `Geometry::from`)"
    )]
    pub fn halfspace(outward_normal: Vec3) -> Self {
        Plane3d::new(outward_normal).into()
    }

    #[deprecated(
        since = "0.5.0",
        note = "please use Bevy's `Capsule3d` primitive (e.g. using `Geometry::from`)"
    )]
    pub fn capsule(half_height: f32, radius: f32) -> Self {
        Capsule3d::new(radius, half_height).into()
    }

    #[deprecated(
        since = "0.5.0",
        note = "please use Bevy's `Cuboid` primitive (e.g. using `Geometry::from`)"
    )]
    pub fn cuboid(half_x: f32, half_y: f32, half_z: f32) -> Self {
        Cuboid::new(half_x * 2., half_y * 2., half_z * 2.).into()
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

    /// Convert bevy's [`Mesh`](Mesh) to [`Geometry`](Geometry), assuming
    /// it's a valid trimesh.
    ///
    /// Also see [`trimesh`](Geometry::trimesh).
    pub fn bevy_trimesh(
        physics: &mut bpx::Physics,
        mesh: &Mesh,
    ) -> Result<Self, TriangleMeshCookingError> {
        let Some((verts, indices)) = extract_mesh_vertices_indices(mesh) else {
            return Err(TriangleMeshCookingError::Failure);
        };
        Self::trimesh(physics, verts.as_slice(), indices.as_slice())
    }

    /// Convert bevy's [`Mesh`](Mesh) to [`Geometry`](Geometry), assuming
    /// it's a convex mesh.
    ///
    /// Also see [`convex_mesh`](Geometry::convex_mesh).
    pub fn bevy_convex_mesh(
        physics: &mut bpx::Physics,
        mesh: &Mesh,
    ) -> Result<Self, ConvexMeshCookingError> {
        let Some((verts, _)) = extract_mesh_vertices_indices(mesh) else {
            return Err(ConvexMeshCookingError::Failure);
        };
        Self::convex_mesh(physics, verts.as_slice())
    }

    /// Apply scale factor to an existing mesh (convex, triangle or heightfield).
    ///
    /// Only applicable to ConvexMesh, TriangleMesh or HeightField.
    ///
    /// Using this function, you can cook a mesh once, then insert it into scene
    /// with different scale factors.
    ///
    ///  - scale - Scaling factor (use `Vec3::splat(2)` to scale up 2x).
    ///  - rotation - The orientation of the scaling axes (usually `Quat::IDENTITY`,
    ///               ignored for HeightField).
    ///
    pub fn with_scale(mut self, scale: Vec3, rotation: Quat) -> Self {
        let new_scale = unsafe { PxMeshScale_new_3(scale.to_physx_sys().as_ptr(), rotation.to_physx().as_ptr()) };
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

    /// Use tighter (but more expensive to compute) bounds around the geometry.
    ///
    /// Only applicable to ConvexMesh, TriangleMesh or HeightField.
    ///
    /// It is enabled by default, use `.with_tight_bounds(false)` to disable.
    ///
    pub fn with_tight_bounds(mut self, tight_bounds: bool) -> Self {
        match &mut self.obj {
            GeometryInner::ConvexMesh { ref mut flags, .. } => {
                flags.set(ConvexMeshGeometryFlags::TightBounds, tight_bounds);
            }
            GeometryInner::TriangleMesh { ref mut flags, .. } => {
                flags.set(MeshGeometryFlags::TightBounds, tight_bounds);
            }
            GeometryInner::HeightField { ref mut flags, .. } => {
                flags.set(MeshGeometryFlags::TightBounds, tight_bounds);
            }
            _ => {
                bevy::log::warn!("unable to set tight bounds, wrong geometry type (not ConvexMesh, TriangleMesh or HeightField)");
            }
        };
        self
    }

    /// Meshes with this flag set are treated as double-sided.
    ///
    /// Only applicable to TriangleMesh or HeightField.
    ///
    /// This flag is currently only used for raycasts and sweeps (it is ignored for overlap queries).
    /// For detailed specifications of this flag for meshes and heightfields please refer to
    /// the Geometry Query section of the PhysX user guide.
    pub fn with_double_sided(mut self, double_sided: bool) -> Self {
        match &mut self.obj {
            GeometryInner::TriangleMesh { ref mut flags, .. } => {
                flags.set(MeshGeometryFlags::DoubleSided, double_sided);
            }
            GeometryInner::HeightField { ref mut flags, .. } => {
                flags.set(MeshGeometryFlags::DoubleSided, double_sided);
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

fn extract_mesh_vertices_indices(mesh: &Mesh) -> Option<(Vec<Vec3>, Vec<[u32; 3]>)> {
    let vertices = mesh.attribute(Mesh::ATTRIBUTE_POSITION)?;
    let indices = mesh.indices()?;

    let vtx: Vec<_> = match vertices {
        VertexAttributeValues::Float32(vtx) => {
            Some(vtx.chunks(3).map(|v| Vec3::new(v[0], v[1], v[2])).collect())
        }
        VertexAttributeValues::Float32x3(vtx) => {
            Some(vtx.iter().map(|v| Vec3::new(v[0], v[1], v[2])).collect())
        }
        _ => None,
    }?;

    let idx = match indices {
        Indices::U16(idx) => idx
            .chunks_exact(3)
            .map(|i| [i[0] as u32, i[1] as u32, i[2] as u32])
            .collect(),
        Indices::U32(idx) => idx.chunks_exact(3).map(|i| [i[0], i[1], i[2]]).collect(),
    };

    Some((vtx, idx))
}
