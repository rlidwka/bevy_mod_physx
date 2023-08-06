use std::mem::MaybeUninit;

use physx::{
    math::{PxBounds3, PxVec3},
    traits::Class,
};

use physx_sys::{
    // TODO: high level wrapper for PxMassProperties
    PxMassProperties,
    PxTriangleMesh_getLocalBounds,
    // TODO: SDF getters
    //PxTriangleMesh_getSDF,
    //PxTriangleMesh_getSDFDimensions,
    //PxTriangleMesh_setPreferSDFProjection_mut,
    //PxTriangleMesh_getPreferSDFProjection,
    PxTriangleMesh_getMassInformation,
    PxTriangleMesh_getNbTriangles,
    PxTriangleMesh_getNbVertices,
    PxTriangleMesh_getTriangleMaterialIndex,
    PxTriangleMesh_getTriangleMeshFlags,
    PxTriangleMesh_getTriangles,
    PxTriangleMesh_getTrianglesRemap,
    PxTriangleMesh_getVertices,
    PxTriangleMesh_getVerticesForModification_mut,
    PxTriangleMesh_refitBVH_mut,
    //PxTriangleMesh_release_mut,
};

pub use physx_sys::{
    PxTriangleMeshFlag as TriangleMeshFlag, PxTriangleMeshFlags as TriangleMeshFlags,
};

pub trait TriangleMeshExtras {
    fn get_nb_vertices(&self) -> u32;
    fn get_vertices(&self) -> &[PxVec3];
    fn get_vertices_for_modification(&mut self) -> &mut [PxVec3];
    fn refit_bvh(&mut self) -> PxBounds3;
    fn get_nb_triangles(&self) -> u32;
    fn get_triangles(&self) -> TriangleMeshIndices<'_>;
    fn get_triangle_mesh_flags(&self) -> TriangleMeshFlags;
    fn get_triangles_remap(&self) -> &[u32];
    fn get_triangle_material_index(&self, triangle_index: u32) -> u16;
    fn get_mass_information(&self) -> PxMassProperties;
    fn get_local_bounds(&self) -> PxBounds3;
}

impl TriangleMeshExtras for physx::triangle_mesh::TriangleMesh {
    /// Returns the number of vertices.
    fn get_nb_vertices(&self) -> u32 {
        unsafe { PxTriangleMesh_getNbVertices(self.as_ptr()) }
    }

    /// Returns the vertices.
    fn get_vertices(&self) -> &[PxVec3] {
        let vertices = unsafe {
            std::slice::from_raw_parts(
                PxTriangleMesh_getVertices(self.as_ptr()),
                self.get_nb_vertices() as usize,
            )
        };

        // SAFETY: PxVec3 is repr(transparent) wrapper of physx_sys::PxVec3
        unsafe { std::mem::transmute(vertices) }
    }

    /// Returns all mesh vertices for modification.
    fn get_vertices_for_modification(&mut self) -> &mut [PxVec3] {
        let vertices = unsafe {
            std::slice::from_raw_parts_mut(
                PxTriangleMesh_getVerticesForModification_mut(self.as_mut_ptr()),
                self.get_nb_vertices() as usize,
            )
        };

        // SAFETY: PxVec3 is repr(transparent) wrapper of physx_sys::PxVec3
        unsafe { std::mem::transmute(vertices) }
    }

    /// Refits BVH for mesh vertices.
    fn refit_bvh(&mut self) -> PxBounds3 {
        unsafe { PxTriangleMesh_refitBVH_mut(self.as_mut_ptr()) }.into()
    }

    /// Returns the number of triangles.
    fn get_nb_triangles(&self) -> u32 {
        unsafe { PxTriangleMesh_getNbTriangles(self.as_ptr()) }
    }

    /// Returns the triangle indices.
    fn get_triangles(&self) -> TriangleMeshIndices<'_> {
        let buffer = unsafe { PxTriangleMesh_getTriangles(self.as_ptr()) };
        let length = self.get_nb_triangles() as usize * 3;

        if self
            .get_triangle_mesh_flags()
            .contains(TriangleMeshFlags::E16BitIndices)
        {
            TriangleMeshIndices::U16(unsafe {
                std::slice::from_raw_parts(buffer as *const u16, length)
            })
        } else {
            TriangleMeshIndices::U32(unsafe {
                std::slice::from_raw_parts(buffer as *const u32, length)
            })
        }
    }

    /// Reads the TriangleMesh flags.
    fn get_triangle_mesh_flags(&self) -> TriangleMeshFlags {
        unsafe { PxTriangleMesh_getTriangleMeshFlags(self.as_ptr()) }
    }

    /// Returns the triangle remapping table.
    fn get_triangles_remap(&self) -> &[u32] {
        unsafe {
            std::slice::from_raw_parts(
                PxTriangleMesh_getTrianglesRemap(self.as_ptr()),
                self.get_nb_triangles() as usize,
            )
        }
    }

    /// Returns material table index of given triangle.
    fn get_triangle_material_index(&self, triangle_index: u32) -> u16 {
        unsafe { PxTriangleMesh_getTriangleMaterialIndex(self.as_ptr(), triangle_index) }
    }

    /// Returns the mass properties of the mesh assuming unit density.
    fn get_mass_information(&self) -> PxMassProperties {
        let mut mass = MaybeUninit::uninit();
        let mut local_inertia = MaybeUninit::uninit();
        let mut local_center_of_mass = MaybeUninit::uninit();

        unsafe {
            PxTriangleMesh_getMassInformation(
                self.as_ptr(),
                mass.as_mut_ptr(),
                local_inertia.as_mut_ptr(),
                local_center_of_mass.as_mut_ptr(),
            );

            PxMassProperties {
                inertiaTensor: local_inertia.assume_init(),
                centerOfMass: local_center_of_mass.assume_init(),
                mass: mass.assume_init(),
            }
        }
    }

    /// Returns the local-space (vertex space) AABB from the triangle mesh.
    fn get_local_bounds(&self) -> PxBounds3 {
        unsafe { PxTriangleMesh_getLocalBounds(self.as_ptr()) }.into()
    }
}

#[derive(Debug)]
pub enum TriangleMeshIndices<'a> {
    U16(&'a [u16]),
    U32(&'a [u32]),
}

impl<'a> TriangleMeshIndices<'a> {
    pub fn len(&self) -> usize {
        match self {
            Self::U16(vec) => vec.len(),
            Self::U32(vec) => vec.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
