// Author: Tom Olsson <tom.olsson@embark-studios.com>
// Copyright © 2019, Embark Studios, all rights reserved.
// Created: 11 April 2019

#![warn(clippy::all)]

/*!

*/
use std::ffi::c_void;

use crate::{
    DeriveClassForNewType,
    cooking::PxHeightFieldDesc,
    math::PxVec3,
    owner::Owner,
    traits::Class,
};

use enumflags2::bitflags;

use physx_sys::{
    PxHeightFieldFlags,
    PxHeightFieldSample,
    PxHeightField_release_mut,
    PxHeightField_saveCells,
    PxHeightField_modifySamples_mut,
    PxHeightField_getNbRows,
    PxHeightField_getNbColumns,
    PxHeightField_getFormat,
    PxHeightField_getSampleStride,
    PxHeightField_getConvexEdgeThreshold,
    PxHeightField_getFlags,
    PxHeightField_getHeight,
    //PxHeightField_getReferenceCount,
    //PxHeightField_acquireReference_mut,
    PxHeightField_getTriangleMaterialIndex,
    PxHeightField_getTriangleNormal,
    PxHeightField_getSample,
    PxHeightField_getTimestamp,
    //PxHeightField_getConcreteTypeName,
};

pub const HEIGHT_SCALE: f32 = 1.0;
pub const XZ_SCALE: f32 = 100.0;

#[repr(transparent)]
pub struct HeightField {
    obj: physx_sys::PxHeightField,
}

DeriveClassForNewType!(HeightField: PxHeightField, PxBase);

impl HeightField {
    /// # Safety
    /// Owner's own the pointer they wrap, using the pointer after dropping the Owner,
    /// or creating multiple Owners from the same pointer will cause UB.  Use `into_ptr` to
    /// retrieve the pointer and consume the Owner without dropping the pointee.
    pub(crate) unsafe fn from_raw(ptr: *mut physx_sys::PxHeightField) -> Option<Owner<Self>> {
        Owner::from_raw(ptr as *mut Self)
    }

    /// Writes out the sample data array.
    pub fn save_cells(&self) -> Vec<PxHeightFieldSample> {
        let buffer_size = self.get_nb_columns() * self.get_nb_rows();
        let byte_size = buffer_size * std::mem::size_of::<PxHeightFieldSample>() as u32;
        let mut samples = Vec::with_capacity(buffer_size as usize);

        unsafe {
            assert_eq!(PxHeightField_saveCells(self.as_ptr(), samples.as_mut_ptr() as *mut c_void, byte_size), byte_size);

            // SAFETY: call above should populate all the values, as verified by the assertion
            samples.set_len(buffer_size as usize)
        }

        samples
    }

    /// Replaces a rectangular subfield in the sample data array.
    pub fn modify_samples(&mut self, start_col: i32, start_row: i32, subfield_desc: &PxHeightFieldDesc, shrink_bounds: bool) -> bool {
        unsafe { PxHeightField_modifySamples_mut(self.as_mut_ptr(), start_col, start_row, subfield_desc.as_ptr(), shrink_bounds) }
    }

    /// Retrieves the number of sample rows in the samples array.
    pub fn get_nb_rows(&self) -> u32 {
        unsafe { PxHeightField_getNbRows(self.as_ptr()) }
    }

    /// Retrieves the number of sample columns in the samples array.
    pub fn get_nb_columns(&self) -> u32 {
        unsafe { PxHeightField_getNbColumns(self.as_ptr()) }
    }

    /// Retrieves the format of the sample data.
    pub fn get_format(&self) -> HeightFieldFormat {
        unsafe { PxHeightField_getFormat(self.as_ptr()) }.into()
    }

    /// Retrieves the offset in bytes between consecutive samples in the array.
    pub fn get_sample_stride(&self) -> u32 {
        unsafe { PxHeightField_getSampleStride(self.as_ptr()) }
    }

    /// Retrieves the convex edge threshold.
    pub fn get_convex_edge_threshold(&self) -> f32 {
        unsafe { PxHeightField_getConvexEdgeThreshold(self.as_ptr()) }
    }

    /// Retrieves the flags bits, combined from values of the enum PxHeightFieldFlag.
    pub fn get_flags(&self) -> PxHeightFieldFlags {
        // TODO: move to rust bitflags
        unsafe { PxHeightField_getFlags(self.as_ptr()) }
    }

    /// Retrieves the height at the given coordinates in grid space.
    pub fn get_height(&self, x: f32, z: f32) -> f32 {
        unsafe { PxHeightField_getHeight(self.as_ptr(), x, z) }
    }

    /// Returns material table index of given triangle.
    pub fn get_triangle_material_index(&self, triangle_index: u32) -> u16 {
        unsafe { PxHeightField_getTriangleMaterialIndex(self.as_ptr(), triangle_index) }
    }

    /// Returns a triangle face normal for a given triangle index.
    pub fn get_triangle_normal(&self, triangle_index: u32) -> PxVec3 {
        unsafe { PxHeightField_getTriangleNormal(self.as_ptr(), triangle_index) }.into()
    }

    /// Returns heightfield sample of given row and column.
    pub fn get_sample(&self, row: u32, column: u32) -> Option<&PxHeightFieldSample> {
        // need to do bound checks, otherwise C++ code will crash with assertion error
        if row < self.get_nb_rows() || column < self.get_nb_columns() {
            Some(unsafe { &*PxHeightField_getSample(self.as_ptr(), row, column) })
        } else {
            None
        }
    }

    /// Returns the number of times the heightfield data has been modified.
    pub fn get_timestamp(&self) -> u32 {
        unsafe { PxHeightField_getTimestamp(self.as_ptr()) }
    }
}

unsafe impl Send for HeightField {}
unsafe impl Sync for HeightField {}

impl Drop for HeightField {
    fn drop(&mut self) {
        unsafe { PxHeightField_release_mut(self.as_mut_ptr()) }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
#[repr(u32)]
pub enum HeightFieldFormat {
    S16tm = 1,
}

impl From<HeightFieldFormat> for physx_sys::PxHeightFieldFormat::Enum {
    fn from(value: HeightFieldFormat) -> Self {
        value as u32
    }
}

impl From<physx_sys::PxHeightFieldFormat::Enum> for HeightFieldFormat {
    fn from(ty: physx_sys::PxHeightFieldFormat::Enum) -> Self {
        match ty {
            1 => Self::S16tm,
            _ => panic!("invalid enum variant"),
        }
    }
}

#[bitflags]
#[derive(Debug, Copy, Clone)]
#[repr(u16)]
pub enum HeightFieldFlag {
    NoboundaryEdges = 1,
}
