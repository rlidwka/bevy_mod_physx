use std::mem::ManuallyDrop;
use std::ptr::drop_in_place;
use std::ptr::null_mut;

use bevy::prelude::*;
use physx::prelude::*;
use physx::traits::Class;
use physx::vehicles::*;

use crate::components::RigidDynamicHandle;
use crate::prelude as bpx;

use physx_sys::{
    PxBatchQuery,
    PxFilterData,
    PxHitFlags,
    PxQueryHit,
    PxQueryHitType,
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
    phys_PxVehicleUpdates,
};

use crate::PhysicsTime;
use crate::PxScene;
use super::Vehicle;
use super::VehicleHandle;

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
    // TODO: this is leaked to prevent crashes due to bevy load order
    friction_pairs: ManuallyDrop<Owner<VehicleDrivableSurfaceToTireFrictionPairs>>,
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
            //unsafe { drop_in_place(self.batch_query); }
        }
    }
}

impl Default for VehicleSimulation {
    fn default() -> Self {
        Self {
            friction_pairs: ManuallyDrop::new(VehicleDrivableSurfaceToTireFrictionPairs::allocate(0, 0).unwrap()),
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
            friction_pairs: ManuallyDrop::new(VehicleDrivableSurfaceToTireFrictionPairs::allocate(0, 0).unwrap()),
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

    pub fn alloc(&mut self, scene: &mut PxScene, max_num_wheels: usize) {
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

    pub fn simulate(&mut self, scene: &mut PxScene, delta: f32, vehicles: &mut [*mut PxVehicleWheels], wheel_count: usize) {
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
        self.friction_pairs = ManuallyDrop::new(friction_pairs);
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

pub fn scene_simulate_vehicles(
    mut scene: ResMut<bpx::Scene>,
    time: Res<PhysicsTime>,
    mut vehicle_simulation: ResMut<VehicleSimulation>,
    mut vehicle_query: Query<&mut VehicleHandle>,
) {
    let delta = time.delta_seconds;
    let mut vehicles = vec![];
    let mut wheel_count = 0;

    for mut vehicle in vehicle_query.iter_mut() {
        match vehicle.as_mut() {
            VehicleHandle::NoDrive(vehicle) => {
                let mut vehicle = vehicle.get_mut(&mut scene);
                wheel_count += vehicle.wheels_sim_data().get_nb_wheels() as usize;
                vehicles.push(vehicle.as_mut_ptr());
            }
            VehicleHandle::Drive4W(vehicle) => {
                let mut vehicle = vehicle.get_mut(&mut scene);
                wheel_count += vehicle.wheels_sim_data().get_nb_wheels() as usize;
                vehicles.push(vehicle.as_mut_ptr());
            }
            VehicleHandle::DriveNW(vehicle) => {
                let mut vehicle = vehicle.get_mut(&mut scene);
                wheel_count += vehicle.wheels_sim_data().get_nb_wheels() as usize;
                vehicles.push(vehicle.as_mut_ptr());
            }
            VehicleHandle::DriveTank(vehicle) => {
                let mut vehicle = vehicle.get_mut(&mut scene);
                wheel_count += vehicle.wheels_sim_data().get_nb_wheels() as usize;
                vehicles.push(vehicle.as_mut_ptr());
            }
        }
    }

    let mut scene = scene.get_mut();
    vehicle_simulation.simulate(&mut scene, delta, vehicles.as_mut(), wheel_count);
}

pub fn create_vehicle_actors(
    mut commands: Commands,
    mut physics: ResMut<bpx::Physics>,
    mut scene: ResMut<bpx::Scene>,
    mut new_actors: Query<(Entity, &mut RigidDynamicHandle, &mut Vehicle), Without<VehicleHandle>>,
) {
    for (entity, mut actor, mut vehicle) in new_actors.iter_mut() {
        let mut actor = actor.get_mut(&mut scene);

        commands.entity(entity)
            .insert(VehicleHandle::new(&mut vehicle, &mut physics, &mut actor));
    }
}
