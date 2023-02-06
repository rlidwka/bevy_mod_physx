use bevy::prelude::*;
use physx::cooking::{PxCooking, PxCookingParams};
use physx::prelude::*;
use physx::traits::Class;
use physx::vehicles::{vehicle_set_basis_vectors, vehicle_set_update_mode, VehicleUpdateMode, VehicleDrivableSurfaceToTireFrictionPairs, VehicleDrivableSurfaceType};
use physx_sys::{
    PxBatchQueryDesc_new, PxFilterData, PxQueryHitType,
    PxScene_createBatchQuery_mut, PxBatchQuery, PxSweepHit, PxSweepQueryResult
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
pub struct VehicleSceneQueryData {
    current_size: usize,
    //raycast_results: Vec<u8>,
    //raycast_hit_buffer: Vec<u8>,
    sweep_results: Vec<u8>,
    sweep_hit_buffer: Vec<u8>,
    batch_query: *mut PxBatchQuery,
}

unsafe impl Send for VehicleSceneQueryData {}
unsafe impl Sync for VehicleSceneQueryData {}

impl VehicleSceneQueryData {
    pub fn alloc(&mut self, scene: &mut Scene, max_num_wheels: usize) {
        unsafe extern "C" fn pre_filter_shader(_data0: &PxFilterData, data1: &PxFilterData/*, _cblock: c_void, _cblocksize: u32, _flags: PxHitFlags*/) -> u32 {
            if 0 == (data1.word3 & 0xffff0000) {
                PxQueryHitType::eNONE
            } else {
                PxQueryHitType::eBLOCK
            }
        }

        // buffers already allocated
        if max_num_wheels <= self.current_size { return; }

        self.current_size = max_num_wheels.next_power_of_two();

        const QUERY_HITS_PER_WHEEL: usize = 1;
        let max_num_hit_points = self.current_size * QUERY_HITS_PER_WHEEL;

        // PxRaycastQueryResult, rust port generates wrong struct size; 80 bytes isn't enough?
        //self.raycast_results = vec![0u8; 100 * self.current_size];
        //self.raycast_hit_buffer = vec![0u8; std::mem::size_of::<PxRaycastHit>() * max_num_hit_points];
        // PxSweepQueryResult, rust port generates wrong struct size; 64 bytes isn't enough?
        self.sweep_results = vec![0u8; 100 * self.current_size];
        self.sweep_hit_buffer = vec![0u8; std::mem::size_of::<PxSweepHit>() * max_num_hit_points];

        let mut sq_desc = unsafe { PxBatchQueryDesc_new(self.current_size as u32, self.current_size as u32, 0) };

        //sq_desc.queryMemory.userRaycastResultBuffer = self.raycast_results.as_mut_ptr() as *mut PxRaycastQueryResult;
        //sq_desc.queryMemory.userRaycastTouchBuffer = self.raycast_hit_buffer.as_mut_ptr() as *mut PxRaycastHit;
        //sq_desc.queryMemory.raycastTouchBufferSize = self.current_size as u32 * max_num_hit_points as u32;

        sq_desc.queryMemory.userSweepResultBuffer = self.sweep_results.as_mut_ptr() as *mut PxSweepQueryResult;
        sq_desc.queryMemory.userSweepTouchBuffer = self.sweep_hit_buffer.as_mut_ptr() as *mut PxSweepHit;
        sq_desc.queryMemory.sweepTouchBufferSize = self.current_size as u32 * max_num_hit_points as u32;

        sq_desc.preFilterShader = pre_filter_shader as *mut c_void;

        if !self.batch_query.is_null() {
            unsafe { drop_in_place(self.batch_query); }
        }

        self.batch_query = unsafe {
            PxScene_createBatchQuery_mut(scene.as_mut_ptr(), &sq_desc as *const _)
        };
    }

    pub fn get_batch_query(&mut self) -> *mut PxBatchQuery {
        self.batch_query
    }

    //pub fn get_raycast_query_buffer(&mut self) -> *mut PxRaycastQueryResult {
    //    self.raycast_results.as_mut_ptr() as *mut PxRaycastQueryResult
    //}

    pub fn get_sweep_query_buffer(&mut self) -> *mut PxSweepQueryResult {
        self.sweep_results.as_mut_ptr() as *mut PxSweepQueryResult
    }
}

impl Default for VehicleSceneQueryData {
    fn default() -> Self {
        Self {
            current_size: 0,
            //raycast_results: vec![],
            //raycast_hit_buffer: vec![],
            sweep_results: vec![],
            sweep_hit_buffer: vec![],
            batch_query: null_mut(),
        }
    }
}

impl Drop for VehicleSceneQueryData {
    fn drop(&mut self) {
        if !self.batch_query.is_null() {
            unsafe { drop_in_place(self.batch_query); }
        }
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct VehicleFrictionPairs(Owner<VehicleDrivableSurfaceToTireFrictionPairs>);

unsafe impl Send for VehicleFrictionPairs {}
unsafe impl Sync for VehicleFrictionPairs {}

impl VehicleFrictionPairs {
    pub fn setup(
        &mut self,
        nb_tire_types: u32,
        nb_surface_types: u32,
        drivable_surface_materials: &[&impl physx::material::Material],
        drivable_surface_types: &[VehicleDrivableSurfaceType],
    ) {
        self.0 = VehicleDrivableSurfaceToTireFrictionPairs::new(
            nb_tire_types,
            nb_surface_types,
            drivable_surface_materials,
            drivable_surface_types,
        ).unwrap();
    }
}

impl Default for VehicleFrictionPairs {
    fn default() -> Self {
        Self(VehicleDrivableSurfaceToTireFrictionPairs::allocate(0, 0).unwrap())
    }
}
