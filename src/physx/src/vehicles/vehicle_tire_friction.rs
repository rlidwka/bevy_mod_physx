use crate::{
    DeriveClassForNewType,
    material::Material,
    owner::Owner,
    traits::Class,
};

use physx_sys::{
    PxVehicleDrivableSurfaceType,
    PxVehicleDrivableSurfaceToTireFrictionPairs_allocate_mut,
    PxVehicleDrivableSurfaceToTireFrictionPairs_setup_mut,
    PxVehicleDrivableSurfaceToTireFrictionPairs_release_mut,
    PxVehicleDrivableSurfaceToTireFrictionPairs_setTypePairFriction_mut,
    PxVehicleDrivableSurfaceToTireFrictionPairs_getTypePairFriction,
    PxVehicleDrivableSurfaceToTireFrictionPairs_getMaxNbSurfaceTypes,
    PxVehicleDrivableSurfaceToTireFrictionPairs_getMaxNbTireTypes,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct VehicleDrivableSurfaceType(pub u32);

/// Driving surface type. Each PxMaterial is associated with a corresponding PxVehicleDrivableSurfaceType.
impl VehicleDrivableSurfaceType {
    pub const SURFACE_TYPE_UNKNOWN: Self = VehicleDrivableSurfaceType(0xffffffff);
}

impl From<PxVehicleDrivableSurfaceType> for VehicleDrivableSurfaceType {
    fn from(value: PxVehicleDrivableSurfaceType) -> Self {
        Self(value.mType)
    }
}

impl From<VehicleDrivableSurfaceType> for PxVehicleDrivableSurfaceType {
    fn from(value: VehicleDrivableSurfaceType) -> Self {
        PxVehicleDrivableSurfaceType { mType: value.0 }
    }
}

impl From<u32> for VehicleDrivableSurfaceType {
    fn from(value: u32) -> Self {
        VehicleDrivableSurfaceType(value)
    }
}

impl From<VehicleDrivableSurfaceType> for u32 {
    fn from(value: VehicleDrivableSurfaceType) -> Self {
        value.0
    }
}

/// Friction for each combination of driving surface type and tire type.
#[repr(transparent)]
pub struct VehicleDrivableSurfaceToTireFrictionPairs {
    obj: physx_sys::PxVehicleDrivableSurfaceToTireFrictionPairs,
}

DeriveClassForNewType!(VehicleDrivableSurfaceToTireFrictionPairs: PxVehicleDrivableSurfaceToTireFrictionPairs);

impl VehicleDrivableSurfaceToTireFrictionPairs {
    /// Set up a PxVehicleDrivableSurfaceToTireFrictionPairs instance for combinations of nbTireTypes tire types and nbSurfaceTypes surface types.
    pub fn new(
        nb_tire_types: u32,
        nb_surface_types: u32,
        drivable_surface_materials: &[&impl Material],
        drivable_surface_types: &[VehicleDrivableSurfaceType],
    ) -> Option<Owner<Self>> {
        let mut result = Self::allocate(nb_tire_types, nb_surface_types)?;
        result.setup(nb_tire_types, nb_surface_types, drivable_surface_materials, drivable_surface_types);
        Some(result)
    }

    /// Allocate the memory for a PxVehicleDrivableSurfaceToTireFrictionPairs instance that can hold data for combinations of tire type and surface type with up to maxNbTireTypes types of tire and maxNbSurfaceTypes types of surface.
    pub fn allocate(max_nb_tire_types: u32, max_nb_surface_types: u32) -> Option<Owner<Self>> {
        unsafe {
            VehicleDrivableSurfaceToTireFrictionPairs::from_raw(
                PxVehicleDrivableSurfaceToTireFrictionPairs_allocate_mut(max_nb_tire_types, max_nb_surface_types)
            )
        }
    }

    /// Set up a PxVehicleDrivableSurfaceToTireFrictionPairs instance for combinations of nbTireTypes tire types and nbSurfaceTypes surface types.
    pub fn setup(
        &mut self,
        nb_tire_types: u32,
        nb_surface_types: u32,
        drivable_surface_materials: &[&impl Material],
        drivable_surface_types: &[VehicleDrivableSurfaceType],
    ) {
        // reinterpreting existing data by adding `as *mut *const _` avoids a copy, not sure if it's sound
        let mut drivable_surface_materials = drivable_surface_materials.iter().map(|x| x.as_ptr()).collect::<Vec<_>>();
        let drivable_surface_types = drivable_surface_types.iter().map(|x| (*x).into()).collect::<Vec<_>>();

        unsafe {
            PxVehicleDrivableSurfaceToTireFrictionPairs_setup_mut(
                self.as_mut_ptr(),
                nb_tire_types,
                nb_surface_types,
                drivable_surface_materials.as_mut_ptr(),
                drivable_surface_types.as_ptr(),
            );
        }
    }

    /// Create a new Owner wrapper around a raw pointer.
    /// # Safety
    /// Owner's own the pointer they wrap, using the pointer after dropping the Owner,
    /// or creating multiple Owners from the same pointer will cause UB.  Use `into_ptr` to
    /// retrieve the pointer and consume the Owner without dropping the pointee.
    /// Initializes user data.
    unsafe fn from_raw(
        ptr: *mut physx_sys::PxVehicleDrivableSurfaceToTireFrictionPairs,
    ) -> Option<Owner<Self>> {
        Owner::from_raw(ptr as *mut Self)
    }

    /// Set the friction for a specified pair of tire type and drivable surface type.
    pub fn set_type_pair_friction(&mut self, surface_type: u32, tire_type: u32, value: f32) {
        unsafe { PxVehicleDrivableSurfaceToTireFrictionPairs_setTypePairFriction_mut(self.as_mut_ptr(), surface_type, tire_type, value) }
    }

    /// Return the friction for a specified combination of surface type and tire type.
    pub fn get_type_pair_friction(&self, surface_type: u32, tire_type: u32) -> f32 {
        unsafe { PxVehicleDrivableSurfaceToTireFrictionPairs_getTypePairFriction(self.as_ptr(), surface_type, tire_type) }
    }

    /// Return the maximum number of surface types.
    pub fn get_max_nb_surface_types(&self) -> u32 {
        unsafe { PxVehicleDrivableSurfaceToTireFrictionPairs_getMaxNbSurfaceTypes(self.as_ptr()) }
    }

    /// Return the maximum number of tire types.
    pub fn get_max_nb_tire_types(&self) -> u32 {
        unsafe { PxVehicleDrivableSurfaceToTireFrictionPairs_getMaxNbTireTypes(self.as_ptr()) }
    }
}

impl Drop for VehicleDrivableSurfaceToTireFrictionPairs {
    fn drop(&mut self) {
        unsafe { PxVehicleDrivableSurfaceToTireFrictionPairs_release_mut(self.as_mut_ptr()) }
    }
}
