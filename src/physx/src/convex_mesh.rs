use std::mem::MaybeUninit;

use crate::{
    DeriveClassForNewType,
    math::{PxVec3, PxBounds3},
    owner::Owner,
    traits::Class,
};

use physx_sys::{
    PxHullPolygon,
    PxConvexMesh_getNbVertices,
    PxConvexMesh_getVertices,
    PxConvexMesh_getIndexBuffer,
    PxConvexMesh_getNbPolygons,
    PxConvexMesh_getPolygonData,
    PxConvexMesh_release_mut,
    //PxConvexMesh_getReferenceCount,
    //PxConvexMesh_acquireReference_mut,
    PxConvexMesh_getMassInformation,
    PxConvexMesh_getLocalBounds,
    //PxConvexMesh_getConcreteTypeName,
    PxConvexMesh_isGpuCompatible,
};

/// A convex mesh.
#[repr(transparent)]
pub struct ConvexMesh {
    obj: physx_sys::PxConvexMesh,
}

DeriveClassForNewType!(ConvexMesh: PxConvexMesh, PxBase);

impl ConvexMesh {
    /// # Safety
    /// Owner's own the pointer they wrap, using the pointer after dropping the Owner,
    /// or creating multiple Owners from the same pointer will cause UB.  Use `into_ptr` to
    /// retrieve the pointer and consume the Owner without dropping the pointee.
    pub unsafe fn from_raw(ptr: *mut physx_sys::PxConvexMesh) -> Option<Owner<ConvexMesh>> {
        Owner::from_raw(ptr as *mut Self)
    }

    /// Returns the number of vertices.
    pub fn get_nb_vertices(&self) -> u32 {
        unsafe { PxConvexMesh_getNbVertices(self.as_ptr()) }
    }

    /// Returns the vertices.
    pub fn get_vertices(&self) -> &[PxVec3] {
        let vertices = unsafe {
            std::slice::from_raw_parts(
                PxConvexMesh_getVertices(self.as_ptr()),
                self.get_nb_vertices() as usize,
            )
        };

        // SAFETY: PxVec3 is repr(transparent) wrapper of physx_sys::PxVec3
        unsafe { std::mem::transmute(vertices) }
    }

    /// Returns the index buffer.
    pub fn get_index_buffer(&self) -> &[u8] {
        let polygon_count = self.get_nb_polygons();

        // for each polygon index buffer contains its points,
        // so we take last polygon's index offset plus its length to calculate total size
        let index_buffer_length = if polygon_count > 0 {
            let last_polygon = self.get_polygon_data(polygon_count - 1).unwrap();
            last_polygon.index_base as usize + last_polygon.nb_verts as usize
        } else { 0 };

        unsafe {
            std::slice::from_raw_parts(
                PxConvexMesh_getIndexBuffer(self.as_ptr()),
                index_buffer_length,
            )
        }
    }

    /// Returns the number of polygons.
    pub fn get_nb_polygons(&self) -> u32 {
        unsafe { PxConvexMesh_getNbPolygons(self.as_ptr()) }
    }

    /// Returns the polygon data.
    pub fn get_polygon_data(&self, index: u32) -> Option<HullPolygon> {
        let mut polygon = MaybeUninit::uninit();

        if unsafe { PxConvexMesh_getPolygonData(self.as_ptr(), index, polygon.as_mut_ptr()) } {
            Some(unsafe { polygon.assume_init() }.into())
        } else {
            None
        }
    }

    /// Returns the mass properties of the mesh assuming unit density (mass, local_inertia, local_center_of_mass).
    pub fn get_mass_information(&self) -> (f32, physx_sys::PxMat33, PxVec3) {
        // TODO: replace physx_sys::PxMat33 with proper rust type
        let mut mass = MaybeUninit::uninit();
        let mut local_inertia = MaybeUninit::uninit();
        let mut local_center_of_mass = MaybeUninit::uninit();

        unsafe {
            PxConvexMesh_getMassInformation(self.as_ptr(), mass.as_mut_ptr(), local_inertia.as_mut_ptr(), local_center_of_mass.as_mut_ptr());

            (mass.assume_init(), local_inertia.assume_init(), local_center_of_mass.assume_init().into())
        }
    }

    /// Returns the local-space (vertex space) AABB from the convex mesh.
    pub fn get_local_bounds(&self) -> PxBounds3 {
        unsafe { PxConvexMesh_getLocalBounds(self.as_ptr()) }.into()
    }

    /// This method decides whether a convex mesh is gpu compatible. If the total number of vertices are more than 64
    /// or any number of vertices in a polygon is more than 32, or convex hull data was not cooked with GPU data enabled
    /// during cooking or was loaded from a serialized collection, the convex hull is incompatible with GPU collision
    /// detection. Otherwise it is compatible.
    pub fn is_gpu_compatible(&self) -> bool {
        unsafe { PxConvexMesh_isGpuCompatible(self.as_ptr()) }
    }
}

unsafe impl Send for ConvexMesh {}
unsafe impl Sync for ConvexMesh {}

impl Drop for ConvexMesh {
    fn drop(&mut self) {
        unsafe { PxConvexMesh_release_mut(self.as_mut_ptr()) }
    }
}

#[derive(Debug, Clone)]
pub struct HullPolygon {
    /// Plane equation for this polygon.
    pub plane: [f32; 4],
    /// Number of vertices/edges in the polygon.
    pub nb_verts: u16,
    /// Offset in index buffer.
    pub index_base: u16,
}

impl From<PxHullPolygon> for HullPolygon {
    fn from(value: PxHullPolygon) -> Self {
        Self {
            plane: value.mPlane,
            nb_verts: value.mNbVerts,
            index_base: value.mIndexBase,
        }
    }
}

impl From<HullPolygon> for PxHullPolygon {
    fn from(value: HullPolygon) -> Self {
        Self {
            mPlane: value.plane,
            mNbVerts: value.nb_verts,
            mIndexBase: value.index_base,
        }
    }
}
