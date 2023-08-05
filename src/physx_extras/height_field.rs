use std::ffi::c_void;

use physx::{cooking::PxHeightFieldDesc, math::PxVec3, traits::Class};

use physx_sys::{
    PxBitAndByte,
    PxHeightField_getConvexEdgeThreshold,
    PxHeightField_getFlags,
    PxHeightField_getFormat,
    PxHeightField_getHeight,
    PxHeightField_getNbColumns,
    PxHeightField_getNbRows,
    PxHeightField_getSample,
    PxHeightField_getSampleStride,
    PxHeightField_getTimestamp,
    //PxHeightField_getConcreteTypeName,
    PxHeightField_getTriangleMaterialIndex,
    PxHeightField_getTriangleNormal,
    PxHeightField_modifySamples_mut,
    //PxHeightField_release_mut,
    PxHeightField_saveCells,
};

pub use physx_sys::{
    //PxHeightFieldTessFlag as HeightFieldTessFlag,
    PxHeightFieldFlag as HeightFieldFlag,
    PxHeightFieldFlags as HeightFieldFlags,
    //PxHeightFieldMaterial as HeightFieldMaterial,
    PxHeightFieldFormat as HeightFieldFormat,
};

pub const HEIGHT_SCALE: f32 = 1.0;
pub const XZ_SCALE: f32 = 100.0;

pub trait HeightFieldExtras {
    fn save_cells(&self) -> Vec<HeightFieldSample>;
    fn modify_samples(
        &mut self,
        start_col: i32,
        start_row: i32,
        subfield_desc: &PxHeightFieldDesc,
        shrink_bounds: bool,
    ) -> bool;
    fn get_nb_rows(&self) -> u32;
    fn get_nb_columns(&self) -> u32;
    fn get_format(&self) -> HeightFieldFormat;
    fn get_sample_stride(&self) -> u32;
    fn get_convex_edge_threshold(&self) -> f32;
    fn get_flags(&self) -> HeightFieldFlags;
    fn get_height(&self, x: f32, z: f32) -> f32;
    fn get_triangle_material_index(&self, triangle_index: u32) -> u16;
    fn get_triangle_normal(&self, triangle_index: u32) -> PxVec3;
    fn get_sample(&self, row: u32, column: u32) -> Option<&HeightFieldSample>;
    fn get_timestamp(&self) -> u32;
}

impl HeightFieldExtras for physx::height_field::HeightField {
    /// Writes out the sample data array.
    fn save_cells(&self) -> Vec<HeightFieldSample> {
        let buffer_size = self.get_nb_columns() * self.get_nb_rows();
        let byte_size = buffer_size * std::mem::size_of::<HeightFieldSample>() as u32;
        let mut samples = Vec::with_capacity(buffer_size as usize);

        unsafe {
            // SAFETY: HeightFieldSample is repr(transparent) of PxHeightFieldSample
            assert_eq!(
                PxHeightField_saveCells(
                    self.as_ptr(),
                    samples.as_mut_ptr() as *mut c_void,
                    byte_size
                ),
                byte_size
            );

            // SAFETY: call above should populate all the values, as verified by the assertion
            samples.set_len(buffer_size as usize);
        }

        samples
    }

    /// Replaces a rectangular subfield in the sample data array.
    fn modify_samples(
        &mut self,
        start_col: i32,
        start_row: i32,
        subfield_desc: &PxHeightFieldDesc,
        shrink_bounds: bool,
    ) -> bool {
        unsafe {
            PxHeightField_modifySamples_mut(
                self.as_mut_ptr(),
                start_col,
                start_row,
                subfield_desc.as_ptr(),
                shrink_bounds,
            )
        }
    }

    /// Retrieves the number of sample rows in the samples array.
    fn get_nb_rows(&self) -> u32 {
        unsafe { PxHeightField_getNbRows(self.as_ptr()) }
    }

    /// Retrieves the number of sample columns in the samples array.
    fn get_nb_columns(&self) -> u32 {
        unsafe { PxHeightField_getNbColumns(self.as_ptr()) }
    }

    /// Retrieves the format of the sample data.
    fn get_format(&self) -> HeightFieldFormat {
        unsafe { PxHeightField_getFormat(self.as_ptr()) }
    }

    /// Retrieves the offset in bytes between consecutive samples in the array.
    fn get_sample_stride(&self) -> u32 {
        unsafe { PxHeightField_getSampleStride(self.as_ptr()) }
    }

    /// Retrieves the convex edge threshold.
    fn get_convex_edge_threshold(&self) -> f32 {
        unsafe { PxHeightField_getConvexEdgeThreshold(self.as_ptr()) }
    }

    /// Retrieves the flags bits, combined from values of the enum HeightFieldFlag.
    fn get_flags(&self) -> HeightFieldFlags {
        unsafe { PxHeightField_getFlags(self.as_ptr()) }
    }

    /// Retrieves the height at the given coordinates in grid space.
    fn get_height(&self, x: f32, z: f32) -> f32 {
        unsafe { PxHeightField_getHeight(self.as_ptr(), x, z) }
    }

    /// Returns material table index of given triangle.
    fn get_triangle_material_index(&self, triangle_index: u32) -> u16 {
        unsafe { PxHeightField_getTriangleMaterialIndex(self.as_ptr(), triangle_index) }
    }

    /// Returns a triangle face normal for a given triangle index.
    fn get_triangle_normal(&self, triangle_index: u32) -> PxVec3 {
        unsafe { PxHeightField_getTriangleNormal(self.as_ptr(), triangle_index) }.into()
    }

    /// Returns heightfield sample of given row and column.
    fn get_sample(&self, row: u32, column: u32) -> Option<&HeightFieldSample> {
        // need to do bound checks, otherwise C++ code will crash with assertion error
        if row < self.get_nb_rows() || column < self.get_nb_columns() {
            // SAFETY: HeightFieldSample is repr(transparent) of PxHeightFieldSample
            Some(unsafe {
                std::mem::transmute(&*PxHeightField_getSample(self.as_ptr(), row, column))
            })
        } else {
            None
        }
    }

    /// Returns the number of times the heightfield data has been modified.
    fn get_timestamp(&self) -> u32 {
        unsafe { PxHeightField_getTimestamp(self.as_ptr()) }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HeightFieldMaterial(i8);

impl HeightFieldMaterial {
    pub const HOLE: Self = Self(physx_sys::PxHeightFieldMaterial::Hole as _);

    pub const fn new(index: i8) -> Self {
        assert!(
            index >= 0,
            "HeightFieldMaterial index must be in 0x00..0x7F range"
        );
        Self(index)
    }

    pub const fn get(&self) -> i8 {
        self.0
    }
}

#[derive(Copy, Clone)]
#[repr(transparent)]
pub struct HeightFieldSample {
    obj: physx_sys::PxHeightFieldSample,
}

//physx::DeriveClassForNewType!(HeightFieldSample: PxHeightFieldSample);

impl From<physx_sys::PxHeightFieldSample> for HeightFieldSample {
    fn from(value: physx_sys::PxHeightFieldSample) -> Self {
        Self { obj: value }
    }
}

impl From<HeightFieldSample> for physx_sys::PxHeightFieldSample {
    fn from(value: HeightFieldSample) -> Self {
        value.obj
    }
}

impl HeightFieldSample {
    // In PhysX, material can only be in 0x00..0x7F range, high bit has special meaning.
    // In material0 high bit is tessellation flag, in material1 it is reserved.
    // We can use PxHeightFieldSample_tessFlag methods, but it's easier to just do bit arithmetic in Rust.
    const BIT_MASK: u8 = 0x80;

    pub fn new(
        height: i16,
        material0: HeightFieldMaterial,
        material1: HeightFieldMaterial,
        tess_flag: bool,
    ) -> Self {
        let mut result: Self = physx_sys::PxHeightFieldSample {
            height,
            materialIndex0: PxBitAndByte {
                structgen_pad0: [material0.0 as u8],
            },
            materialIndex1: PxBitAndByte {
                structgen_pad0: [material1.0 as u8],
            },
        }
        .into();

        if tess_flag {
            result.set_tess_flag();
        }

        result
    }

    pub fn height(&self) -> i16 {
        self.obj.height
    }

    pub fn set_height(&mut self, height: i16) {
        self.obj.height = height;
    }

    pub fn tess_flag(&self) -> bool {
        // same as PxHeightFieldSample_tessFlag
        self.obj.materialIndex0.structgen_pad0[0] & Self::BIT_MASK != 0
    }

    pub fn set_tess_flag(&mut self) {
        // same as PxHeightFieldSample_setTessFlag_mut
        self.obj.materialIndex0.structgen_pad0[0] |= Self::BIT_MASK;
    }

    pub fn clear_tess_flag(&mut self) {
        // same as PxHeightFieldSample_clearTessFlag_mut
        self.obj.materialIndex0.structgen_pad0[0] &= !Self::BIT_MASK;
    }

    pub fn material0(&self) -> HeightFieldMaterial {
        HeightFieldMaterial::new(
            (self.obj.materialIndex0.structgen_pad0[0] & !Self::BIT_MASK) as i8,
        )
    }

    pub fn material1(&self) -> HeightFieldMaterial {
        HeightFieldMaterial::new(
            (self.obj.materialIndex1.structgen_pad0[0] & !Self::BIT_MASK) as i8,
        )
    }

    pub fn set_material0(&mut self, material: HeightFieldMaterial) {
        self.obj.materialIndex0.structgen_pad0[0] =
            material.get() as u8 | self.obj.materialIndex0.structgen_pad0[0] & Self::BIT_MASK;
    }

    pub fn set_material1(&mut self, material: HeightFieldMaterial) {
        self.obj.materialIndex1.structgen_pad0[0] =
            material.get() as u8 | self.obj.materialIndex1.structgen_pad0[0] & Self::BIT_MASK;
    }
}
