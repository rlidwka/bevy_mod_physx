use bevy::prelude::*;
use physx::cooking::{PxCooking, PxCookingParams};
use physx::prelude::*;
use physx::traits::Class;
use physx::vehicles::{
    VehicleDrivableSurfaceToTireFrictionPairs,
    vehicle_set_basis_vectors,
    vehicle_set_max_hit_actor_acceleration,
    vehicle_set_sweep_hit_rejection_angles,
    vehicle_set_update_mode,
};
use physx_sys::{
    PxBatchQuery,
    PxRaycastHit,
    PxRaycastQueryResult,
    PxSweepHit,
    PxSweepQueryResult,
    PxVehicleWheels,
    PxBatchQueryDesc_new,
    PxScene_createBatchQuery_mut,
    PxScene_getGravity,
    phys_PxVehicleSuspensionRaycasts,
    phys_PxVehicleSuspensionSweeps,
    phys_PxVehicleUpdates, PxQueryHitType, PxFilterData, PxHitFlags, PxQueryHit,
};
use std::ptr::{null_mut, drop_in_place};

use crate::{FoundationDescriptor, SceneDescriptor};

use super::prelude::*;
use super::prelude as bpx;
use super::{PxShape, PxScene};

struct ErrorCallback;

impl physx::physics::ErrorCallback for ErrorCallback {
    fn report_error(
        &self,
        code: enumflags2::BitFlags<physx::foundation::ErrorCode>,
        message: &str,
        file: &str,
        line: u32,
    ) {
        bevy::log::error!(target: "bevy_physx", "[{file:}:{line:}] {code:40}: {message:}");
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct Physics(PhysicsFoundation<physx::foundation::DefaultAllocator, PxShape>);

impl Physics {
    pub fn new(foundation_desc: &FoundationDescriptor) -> Self {
        let mut builder = physx::physics::PhysicsFoundationBuilder::default();
        builder.enable_visual_debugger(foundation_desc.visual_debugger);
        builder.with_extensions(foundation_desc.extensions);
        builder.with_vehicle_sdk(foundation_desc.vehicles);
        builder.set_pvd_port(foundation_desc.visual_debugger_port);
        if let Some(host) = foundation_desc.visual_debugger_remote.as_ref() {
            builder.set_pvd_host(host);
        }
        builder.set_length_tolerance(foundation_desc.tolerances.length);
        builder.set_speed_tolerance(foundation_desc.tolerances.speed);
        builder.with_error_callback(ErrorCallback);

        let physics = builder.build();

        if physics.is_none() && foundation_desc.visual_debugger {
            // failed to connect, try without debugger
            let mut without_debugger = foundation_desc.clone();
            without_debugger.visual_debugger = false;
            return Self::new(&without_debugger);
        }

        let physics = physics.expect("building PhysX foundation failed");

        if foundation_desc.vehicles {
            vehicle_set_basis_vectors(
                foundation_desc.vehicles_basis_vectors[0].to_physx(),
                foundation_desc.vehicles_basis_vectors[1].to_physx(),
            );
            vehicle_set_update_mode(foundation_desc.vehicles_update_mode);
            vehicle_set_max_hit_actor_acceleration(foundation_desc.vehicles_max_hit_actor_acceleration);
            vehicle_set_sweep_hit_rejection_angles(
                foundation_desc.vehicles_sweep_hit_rejection_angles[0],
                foundation_desc.vehicles_sweep_hit_rejection_angles[1],
            );
        }

        Self(physics)
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct Scene(Owner<PxScene>);

impl Scene {
    pub fn new(physics: &mut Physics, d: &SceneDescriptor) -> Self {
        use physx::physics::Physics; // physx trait clashes with our wrapper

        // PxBounds3 doesn't have Clone/Copy, even though it should
        let sanity_bounds = unsafe {
            physx::math::PxBounds3::from(*(std::mem::transmute::<_, &physx_sys::PxBounds3>(&d.sanity_bounds)))
        };

        // not needless match, as it doesn't support Clone/Copy
        #[allow(clippy::needless_match)]
        let simulation_filter_shader = match d.simulation_filter_shader {
            FilterShaderDescriptor::Default => FilterShaderDescriptor::Default,
            FilterShaderDescriptor::Custom(f) => FilterShaderDescriptor::Custom(f),
            FilterShaderDescriptor::CallDefaultFirst(f) => FilterShaderDescriptor::CallDefaultFirst(f),
        };

        let scene = physics
            .create(physx::traits::descriptor::SceneDescriptor {
                gravity: d.gravity.to_physx(),
                kine_kine_filtering_mode: d.kine_kine_filtering_mode,
                static_kine_filtering_mode: d.static_kine_filtering_mode,
                broad_phase_type: d.broad_phase_type,
                limits: d.limits,
                friction_type: d.friction_type,
                solver_type: d.solver_type,
                bounce_threshold_velocity: d.bounce_threshold_velocity,
                friction_offset_threshold: d.friction_offset_threshold,
                ccd_max_separation: d.ccd_max_separation,
                solver_offset_slop: d.solver_offset_slop,
                flags: d.flags,
                static_structure: d.static_structure,
                dynamic_structure: d.dynamic_structure,
                dynamic_tree_rebuild_rate_hint: d.dynamic_tree_rebuild_rate_hint,
                scene_query_update_mode: d.scene_query_update_mode,
                solver_batch_size: d.solver_batch_size,
                solver_articulation_batch_size: d.solver_articulation_batch_size,
                nb_contact_data_blocks: d.nb_contact_data_blocks,
                max_nb_contact_data_blocks: d.max_nb_contact_data_blocks,
                max_bias_coefficient: d.max_bias_coefficient,
                contact_report_stream_buffer_size: d.contact_report_stream_buffer_size,
                ccd_max_passes: d.ccd_max_passes,
                ccd_threshold: d.ccd_threshold,
                wake_counter_reset_value: d.wake_counter_reset_value,
                sanity_bounds,
                simulation_filter_shader,
                thread_count: d.thread_count,
                gpu_max_num_partitions: d.gpu_max_num_partitions,
                gpu_compute_version: d.gpu_compute_version,
                ..physx::traits::descriptor::SceneDescriptor::new(())
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

#[derive(Default, Debug, Clone, Copy)]
pub enum VehicleSimulationMethod {
    #[default]
    Raycast,
    Sweep {
        nb_hits_per_query: u16,
        sweep_width_scale: f32,
        sweep_radius_scale: f32,
    },
}

#[derive(Resource)]
pub struct VehicleSimulation {
    friction_pairs: Owner<VehicleDrivableSurfaceToTireFrictionPairs>,
    current_size: usize,
    simulation_method: VehicleSimulationMethod,
    result_buffer: Vec<u8>, // raycast results or sweep results
    hit_buffer: Vec<u8>, // raycast hit buffer or sweep hit buffer
    batch_query: *mut PxBatchQuery,
    pre_filter_shader: Option<for<'a> unsafe extern "C" fn(&'a PxFilterData, &'a PxFilterData, *const std::ffi::c_void, u32, PxHitFlags) -> PxQueryHitType::Enum>,
    post_filter_shader: Option<for<'a> unsafe extern "C" fn(&'a PxFilterData, &'a PxFilterData, *const std::ffi::c_void, u32, &'a PxQueryHit) -> PxQueryHitType::Enum>,
    shader_data: Option<(*mut std::ffi::c_void, u32)>,
}

unsafe impl Send for VehicleSimulation {}
unsafe impl Sync for VehicleSimulation {}

impl Drop for VehicleSimulation {
    fn drop(&mut self) {
        if !self.batch_query.is_null() {
            unsafe { drop_in_place(self.batch_query); }
        }
    }
}

impl Default for VehicleSimulation {
    fn default() -> Self {
        Self {
            friction_pairs: VehicleDrivableSurfaceToTireFrictionPairs::allocate(0, 0).unwrap(),
            current_size: 0,
            simulation_method: default(),
            result_buffer: vec![],
            hit_buffer: vec![],
            batch_query: null_mut(),
            pre_filter_shader: None,
            post_filter_shader: None,
            shader_data: None,
        }
    }
}

impl VehicleSimulation {
    pub fn new(simulation_method: VehicleSimulationMethod) -> Self {
        Self {
            friction_pairs: VehicleDrivableSurfaceToTireFrictionPairs::allocate(0, 0).unwrap(),
            current_size: 0,
            simulation_method,
            result_buffer: vec![],
            hit_buffer: vec![],
            batch_query: null_mut(),
            pre_filter_shader: None,
            post_filter_shader: None,
            shader_data: None,
        }
    }

    pub fn alloc(&mut self, scene: &mut Scene, max_num_wheels: usize) {
        // buffers already allocated
        if max_num_wheels <= self.current_size { return; }

        self.current_size = max_num_wheels.next_power_of_two();

        let use_sweeps;
        let query_hits_per_wheel;

        match self.simulation_method {
            VehicleSimulationMethod::Raycast => {
                use_sweeps = false;
                query_hits_per_wheel = 1;
            }
            VehicleSimulationMethod::Sweep { nb_hits_per_query, .. } => {
                use_sweeps = true;
                query_hits_per_wheel = nb_hits_per_query as usize;
            }
        }

        let max_num_hit_points = self.current_size * query_hits_per_wheel;

        if use_sweeps {
            // PxSweepQueryResult, rust port generates wrong struct size; 64 bytes isn't enough?
            self.result_buffer = vec![0u8; 100 * self.current_size];
            self.hit_buffer = vec![0u8; std::mem::size_of::<PxSweepHit>() * max_num_hit_points];
        } else {
            // PxRaycastQueryResult, rust port generates wrong struct size; 80 bytes isn't enough?
            self.result_buffer = vec![0u8; 100 * self.current_size];
            self.hit_buffer = vec![0u8; std::mem::size_of::<PxRaycastHit>() * max_num_hit_points];
        }

        let mut sq_desc = unsafe { PxBatchQueryDesc_new(self.current_size as u32, self.current_size as u32, 0) };

        if use_sweeps {
            sq_desc.queryMemory.userSweepResultBuffer = self.result_buffer.as_mut_ptr() as *mut PxSweepQueryResult;
            sq_desc.queryMemory.userSweepTouchBuffer = self.hit_buffer.as_mut_ptr() as *mut PxSweepHit;
            sq_desc.queryMemory.sweepTouchBufferSize = self.current_size as u32 * max_num_hit_points as u32;
        } else {
            sq_desc.queryMemory.userRaycastResultBuffer = self.result_buffer.as_mut_ptr() as *mut PxRaycastQueryResult;
            sq_desc.queryMemory.userRaycastTouchBuffer = self.hit_buffer.as_mut_ptr() as *mut PxRaycastHit;
            sq_desc.queryMemory.raycastTouchBufferSize = self.current_size as u32 * max_num_hit_points as u32;
        }

        if let Some(pre_filter_shader) = self.pre_filter_shader {
            sq_desc.preFilterShader = pre_filter_shader as *mut _;
        }
        if let Some(post_filter_shader) = self.post_filter_shader {
            sq_desc.postFilterShader = post_filter_shader as *mut _;
        }
        if let Some(shader_data) = self.shader_data {
            sq_desc.filterShaderData = shader_data.0;
            sq_desc.filterShaderDataSize = shader_data.1;
        }

        if !self.batch_query.is_null() {
            unsafe { drop_in_place(self.batch_query); }
        }

        self.batch_query = unsafe {
            PxScene_createBatchQuery_mut(scene.as_mut_ptr(), &sq_desc as *const _)
        };
    }

    pub fn simulate(&mut self, scene: &mut Scene, delta: f32, vehicles: &mut [*mut PxVehicleWheels], wheel_count: usize) {
        if vehicles.is_empty() { return; }

        self.alloc(scene, wheel_count);

        let gravity = unsafe { PxScene_getGravity(scene.as_ptr()) };

        unsafe {
            match self.simulation_method {
                VehicleSimulationMethod::Raycast => {
                    phys_PxVehicleSuspensionRaycasts(
                        self.batch_query,
                        vehicles.len() as u32,
                        vehicles.as_mut_ptr() as *mut *mut PxVehicleWheels,
                        wheel_count as u32,
                        self.result_buffer.as_mut_ptr() as *mut _,
                        vec![true; vehicles.len()].as_ptr(),
                    );
                }
                VehicleSimulationMethod::Sweep { nb_hits_per_query, sweep_width_scale, sweep_radius_scale } => {
                    phys_PxVehicleSuspensionSweeps(
                        self.batch_query,
                        vehicles.len() as u32,
                        vehicles.as_mut_ptr() as *mut *mut PxVehicleWheels,
                        wheel_count as u32,
                        self.result_buffer.as_mut_ptr() as *mut _,
                        nb_hits_per_query,
                        null_mut(),
                        sweep_width_scale,
                        sweep_radius_scale,
                    );
                }
            }

            phys_PxVehicleUpdates(
                delta,
                gravity.as_ptr(),
                self.friction_pairs.as_ptr(),
                vehicles.len() as u32,
                vehicles.as_mut_ptr() as *mut *mut PxVehicleWheels,
                null_mut(),
                null_mut(),
            );
        }
    }

    pub fn set_friction_pairs(&mut self, friction_pairs: Owner<VehicleDrivableSurfaceToTireFrictionPairs>) {
        self.friction_pairs = friction_pairs;
    }

    pub fn set_filter_shader(
        &mut self,

        pre_filter_shader: Option<for<'a> unsafe extern "C" fn(
            query_filter_data: &'a PxFilterData,
            object_filter_data: &'a PxFilterData,
            shader_data: *const std::ffi::c_void,
            shader_data_size: u32,
            hit_flags: PxHitFlags,
        ) -> PxQueryHitType::Enum>,

        post_filter_shader: Option<for <'a> unsafe extern "C" fn(
            query_filter_data: &'a PxFilterData,
            object_filter_data: &'a PxFilterData,
            shader_data: *const std::ffi::c_void,
            shader_data_size: u32,
            hit: &'a PxQueryHit,
        ) -> PxQueryHitType::Enum>,

        shader_data: Option<(*mut std::ffi::c_void, u32)>,
    ) {
        self.pre_filter_shader = pre_filter_shader;
        self.post_filter_shader = post_filter_shader;
        self.shader_data = shader_data;
        self.current_size = 0; // reset buffers
    }

    pub fn set_collision_method(&mut self, method: VehicleSimulationMethod) {
        self.simulation_method = method;
        self.current_size = 0; // reset buffers
    }

    pub fn get_friction_pairs(&self) -> &VehicleDrivableSurfaceToTireFrictionPairs {
        &self.friction_pairs
    }

    pub fn get_friction_pairs_mut(&mut self) -> &mut VehicleDrivableSurfaceToTireFrictionPairs {
        &mut self.friction_pairs
    }

    pub fn get_collision_method(&self) -> VehicleSimulationMethod {
        self.simulation_method
    }
}
