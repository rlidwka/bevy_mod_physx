use bevy::prelude::*;
use physx::cooking::{PxCooking, PxCookingParams};
use physx::prelude::*;
use physx::traits::Class;
use physx::vehicles::{vehicle_set_basis_vectors, vehicle_set_update_mode, VehicleUpdateMode};
use physx_sys::{
    PxRaycastHit, PxBatchQueryDesc_new, PxRaycastQueryResult, PxFilterData, PxQueryHitType,
    PxScene_createBatchQuery_mut, PxBatchQuery, PxVehicleDrivableSurfaceToTireFrictionPairs,
    PxVehicleDrivableSurfaceToTireFrictionPairs_allocate_mut, PxVehicleDrivableSurfaceType,
    PxVehicleDrivableSurfaceToTireFrictionPairs_setup_mut, PxVehicleDrivableSurfaceToTireFrictionPairs_release_mut,
    PxVehicleDrivableSurfaceToTireFrictionPairs_setTypePairFriction_mut,
    PxVehicleDrivableSurfaceToTireFrictionPairs_getTypePairFriction
};
use std::ffi::c_void;
use std::ptr::{null_mut, drop_in_place};

use super::prelude::*;
use super::prelude as bpx;
use super::{PxShape, PxScene};

#[derive(Resource, Deref, DerefMut)]
pub struct Physics(PhysicsFoundation<physx::foundation::DefaultAllocator, PxShape>);

impl Physics {
    pub fn new(enable_debugger: bool, enable_vsdk: bool) -> Self {
        let mut physics;

        let mut builder = physx::physics::PhysicsFoundationBuilder::default();
        builder.enable_visual_debugger(enable_debugger);
        builder.with_extensions(true);
        builder.with_vehicle_sdk(enable_vsdk);
        physics = builder.build();

        if physics.is_none() && enable_debugger {
            // failed to connect, try without debugger
            let mut builder = physx::physics::PhysicsFoundationBuilder::default();
            builder.with_extensions(true);
            builder.with_vehicle_sdk(enable_vsdk);
            physics = builder.build();
        }

        let physics = physics.expect("building PhysX foundation failed");

        if enable_vsdk {
            vehicle_set_basis_vectors(PxVec3::new(0., 1., 0.), PxVec3::new(0., 0., 1.));
            vehicle_set_update_mode(VehicleUpdateMode::VelocityChange);
        }

        Self(physics)
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct Scene(Owner<PxScene>);

impl Scene {
    pub fn new(physics: &mut Physics, gravity: Vec3) -> Self {
        use physx::physics::Physics; // physx trait clashes with our wrapper

        let scene = physics
            .create(SceneDescriptor {
                gravity: gravity.to_physx(),
                ..SceneDescriptor::new(())
            })
            .unwrap();

        Self(scene)
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct Cooking(Owner<PxCooking>);

impl Cooking {
    pub fn new(physics: &mut Physics) -> Self {
        let params = &PxCookingParams::new(&**physics).expect("failed to create cooking params");
        let cooking = PxCooking::new(physics.foundation_mut(), params).expect("failed to create cooking");
        Self(cooking)
    }
}

#[derive(Resource, Deref, DerefMut, Default)]
pub struct DefaultMaterial(Option<Handle<bpx::Material>>);

#[derive(Resource)]
pub struct VehicleRaycastBuffer {
    current_size: usize,
    sq_results: Vec<u8>,
    sq_hit_buffer: Vec<u8>,
    batch_query: *mut PxBatchQuery,
}

unsafe impl Send for VehicleRaycastBuffer {}
unsafe impl Sync for VehicleRaycastBuffer {}

impl VehicleRaycastBuffer {
    pub fn alloc(&mut self, scene: &mut Scene, wheel_count: usize) {
        extern "C" fn pre_filter_shader(_data0: &PxFilterData, data1: &PxFilterData/*, _cblock: c_void, _cblocksize: u32, _flags: PxHitFlags*/) -> u32 {
            if 0 == (data1.word3 & 0xffff0000) {
                PxQueryHitType::eNONE
            } else {
                PxQueryHitType::eBLOCK
            }
        }

        // buffers already allocated
        if wheel_count <= self.current_size { return; }

        self.current_size = wheel_count.next_power_of_two();
        // PxRaycastQueryResult, rust port generates wrong struct size, also 80 bytes aren't enough?
        self.sq_results = vec![0u8; 100 * self.current_size];
        self.sq_hit_buffer = vec![0u8; std::mem::size_of::<PxRaycastHit>() * self.current_size];
        let mut sq_desc = unsafe { PxBatchQueryDesc_new(self.current_size as u32, 0, 0) };

        if !self.batch_query.is_null() {
            unsafe { drop_in_place(self.batch_query); }
        }

        self.batch_query = unsafe {
            sq_desc.queryMemory.userRaycastResultBuffer = self.sq_results.as_mut_ptr() as *mut PxRaycastQueryResult;
            sq_desc.queryMemory.userRaycastTouchBuffer = self.sq_hit_buffer.as_mut_ptr() as *mut PxRaycastHit;
            sq_desc.queryMemory.raycastTouchBufferSize = self.current_size as u32;
            sq_desc.preFilterShader = pre_filter_shader as *mut c_void;
            PxScene_createBatchQuery_mut(scene.as_mut_ptr(), &sq_desc as *const _)
        };
    }

    pub fn get_batch_query(&mut self) -> *mut PxBatchQuery {
        self.batch_query
    }

    pub fn get_query_results(&mut self) -> *mut PxRaycastQueryResult {
        self.sq_results.as_mut_ptr() as *mut PxRaycastQueryResult
    }
}

impl Default for VehicleRaycastBuffer {
    fn default() -> Self {
        Self {
            current_size: 0,
            sq_results: vec![],
            sq_hit_buffer: vec![],
            batch_query: null_mut(),
        }
    }
}

impl Drop for VehicleRaycastBuffer {
    fn drop(&mut self) {
        if !self.batch_query.is_null() {
            unsafe { drop_in_place(self.batch_query); }
        }
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct VehicleFrictionPairs(
    *mut PxVehicleDrivableSurfaceToTireFrictionPairs
);

unsafe impl Send for VehicleFrictionPairs {}
unsafe impl Sync for VehicleFrictionPairs {}

impl VehicleFrictionPairs {
    pub fn setup(&mut self, drivable_surface_materials: &[&bpx::Material], drivable_surface_types: &[PxVehicleDrivableSurfaceType]) {
        if !self.0.is_null() {
            unsafe { drop_in_place(self.0); }
        }

        self.0 = unsafe {
            PxVehicleDrivableSurfaceToTireFrictionPairs_allocate_mut(
                drivable_surface_materials.len() as u32,
                drivable_surface_types.len() as u32,
            )
        };

        let mut materials_px = drivable_surface_materials.iter().map(|m| (*m).as_ptr()).collect::<Vec<_>>();

        unsafe {
            PxVehicleDrivableSurfaceToTireFrictionPairs_setup_mut(
                self.0,
                drivable_surface_materials.len() as u32,
                drivable_surface_types.len() as u32,
                materials_px.as_mut_ptr(),
                drivable_surface_types.as_ptr(),
            );
        }
    }

    pub fn set_type_pair_friction(&mut self, surface_type: u32, tire_type: u32, value: f32) {
        unsafe {
            PxVehicleDrivableSurfaceToTireFrictionPairs_setTypePairFriction_mut(self.0, surface_type, tire_type, value);
        }
    }

    pub fn get_type_pair_friction(&mut self, surface_type: u32, tire_type: u32) -> f32 {
        unsafe {
            PxVehicleDrivableSurfaceToTireFrictionPairs_getTypePairFriction(self.0, surface_type, tire_type)
        }
    }
}

impl Default for VehicleFrictionPairs {
    fn default() -> Self {
        Self(unsafe { PxVehicleDrivableSurfaceToTireFrictionPairs_allocate_mut(0, 0) })
    }
}

impl Drop for VehicleFrictionPairs {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                PxVehicleDrivableSurfaceToTireFrictionPairs_release_mut(self.0);
                drop_in_place(self.0);
            }
        }
    }
}
