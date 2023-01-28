////////////////////////////////////////////////
// Class
/// Trait for getting raw pointers for FFI calls, used to provide default implementations
/// of traits that "re-object" a C wrapper of a C++ library.
///
/// # Safety
/// **Implementing `Class<S>` for `T` where `S` is not a superclass of `T` will cause Undefined Behaviour.**
///
/// This trait may hide a raw pointer cast from `*Self` to `*T`.  It is intended for use in
/// default implementations for traits wrapping C++ classes.  In C++-land this is just how
/// things are done, but From Rust's perspective, this is madness.
/// The relations defined between types using this trait must align with the C++ class hierarchy.
/// The [Inherit] trait can be used to simplify implementing `Class<S> for T where T: Class<T>`.
pub unsafe trait Class<S> {
    /// Returns a raw const pointer to the wrapped type.
    /// Retrieving a raw pointer is safe.  However, pretty much any use of a raw pointer is unsafe.
    /// In particular: this pointer should not be used to construct a second owning wrapper around the pointer.
    fn as_ptr(&self) -> *const S;

    /// Returns a raw mut pointer to the wrapped type.
    /// Retrieving a raw pointer is safe.  However, pretty much any use of a raw pointer is unsafe.
    /// In particular: this pointer should not be used to construct a second owning wrapper around the pointer.
    fn as_mut_ptr(&mut self) -> *mut S;
}

/// Macro for quickly defining Class<...> impls for new type wrappers.
/// The type must be repr(transparent), and have the Px object in a field
/// named obj.  Will not work if the type parameters have trait bounds.
macro_rules! DeriveClassForNewType {
    ($PxWrap:ident : $($PxClass:ident),+) => {
        $(unsafe impl Class<::physx_sys::$PxClass> for $PxWrap {
            fn as_ptr(&self) -> *const ::physx_sys::$PxClass {
                &self.obj as *const _ as *const _
            }
            fn as_mut_ptr(&mut self) -> *mut ::physx_sys::$PxClass {
                &mut self.obj as *mut _ as *mut _
            }
        })+
    }
}

pub(crate) use DeriveClassForNewType;

/// Derive Class<T> for the raw Px* types.
macro_rules! DeriveClass {
    ($PxType:ty $(: $($PxClass:ty),*)?) => {
        unsafe impl Class<$PxType> for $PxType {
            fn as_ptr(&self) -> *const $PxType {
                self
            }
            fn as_mut_ptr(&mut self) -> *mut $PxType {
                self
            }
        }

        $($(unsafe impl Class<$PxClass> for $PxType {
            fn as_ptr(&self) -> *const $PxClass {
                self as *const _ as *const _
            }
            fn as_mut_ptr(&mut self) -> *mut $PxClass {
                self as *mut _ as *mut _
            }
        })*)?
    }
}

use physx_sys::*;

DeriveClass!(PxBase);
DeriveClass!(PxVehicleAckermannGeometryData);
DeriveClass!(PxVehicleAntiRollBarData);
DeriveClass!(PxVehicleAutoBoxData);
DeriveClass!(PxVehicleClutchData);
DeriveClass!(PxVehicleDifferential4WData);
DeriveClass!(PxVehicleDifferentialNWData);
DeriveClass!(PxVehicleDrive4W: PxVehicleDrive, PxVehicleWheels, PxBase);
DeriveClass!(PxVehicleDrive4WRawInputData);
DeriveClass!(PxVehicleDrive: PxVehicleWheels);
DeriveClass!(PxVehicleDriveDynData);
DeriveClass!(PxVehicleDriveNW: PxVehicleDrive, PxVehicleWheels, PxBase);
DeriveClass!(PxVehicleDriveNWRawInputData: PxVehicleDrive4WRawInputData);
DeriveClass!(PxVehicleDriveSimData);
DeriveClass!(PxVehicleDriveSimData4W: PxVehicleDriveSimData);
DeriveClass!(PxVehicleDriveSimDataNW: PxVehicleDriveSimData);
DeriveClass!(PxVehicleDriveTank: PxVehicleDrive, PxVehicleWheels, PxBase);
DeriveClass!(PxVehicleDriveTankRawInputData);
DeriveClass!(PxVehicleEngineData);
DeriveClass!(PxVehicleGearsData);
DeriveClass!(PxVehicleKeySmoothingData);
DeriveClass!(PxVehicleNoDrive: PxVehicleWheels, PxBase);
DeriveClass!(PxVehiclePadSmoothingData);
DeriveClass!(PxVehicleSuspensionData);
DeriveClass!(PxVehicleTireData);
DeriveClass!(PxVehicleTireLoadFilterData);
DeriveClass!(PxVehicleWheelData);
DeriveClass!(PxVehicleWheels: PxBase);
DeriveClass!(PxVehicleWheelsDynData);
DeriveClass!(PxVehicleWheelsSimData);
