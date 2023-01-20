use bevy::prelude::*;
use physx::cooking::{PxCooking, PxCookingParams};
use physx::prelude::*;
use physx::traits::Class;
use physx_sys::{phys_PxInitVehicleSDK, phys_PxVehicleSetBasisVectors, phys_PxVehicleSetUpdateMode, PxVehicleUpdateMode, phys_PxCloseVehicleSDK};
use std::ops::{Deref, DerefMut};
use std::ptr::null_mut;
use crate::assets::BPxMaterial;

use super::prelude::*;
use super::{PxShape, PxScene};

#[derive(Resource)]
pub struct BPxPhysics {
    physics: PhysicsFoundation<physx::foundation::DefaultAllocator, PxShape>,
    vsdk: bool,
}

impl BPxPhysics {
    pub fn new(enable_debugger: bool, enable_vsdk: bool) -> Self {
        let mut physics;

        let mut builder = physx::physics::PhysicsFoundationBuilder::default();
        builder.enable_visual_debugger(enable_debugger);
        builder.with_extensions(true);
        physics = builder.build();

        if physics.is_none() && enable_debugger {
            // failed to connect, try without debugger
            let mut builder = physx::physics::PhysicsFoundationBuilder::default();
            builder.with_extensions(true);
            physics = builder.build();
        }

        let mut physics = physics.expect("building PhysX foundation failed");

        if enable_vsdk {
            unsafe {
                phys_PxInitVehicleSDK(physics.as_mut_ptr(), null_mut());
                phys_PxVehicleSetBasisVectors(PxVec3::new(0.,1.,0.).as_ptr(), PxVec3::new(0.,0.,1.).as_ptr());
                phys_PxVehicleSetUpdateMode(PxVehicleUpdateMode::eVELOCITY_CHANGE);
            }
        }

        Self { physics, vsdk: enable_vsdk }
    }
}

impl Deref for BPxPhysics {
    type Target = PhysicsFoundation<physx::foundation::DefaultAllocator, PxShape>;
    fn deref(&self) -> &Self::Target {
        &self.physics
    }
}

impl DerefMut for BPxPhysics {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.physics
    }
}

impl Drop for BPxPhysics {
    fn drop(&mut self) {
        if self.vsdk {
            unsafe {
                phys_PxCloseVehicleSDK(null_mut());
            }
            return;
        }
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct BPxScene(Owner<PxScene>);

impl BPxScene {
    pub fn new(physics: &mut BPxPhysics, gravity: Vec3) -> Self {
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
pub struct BPxCooking(Owner<PxCooking>);

impl BPxCooking {
    pub fn new(physics: &mut BPxPhysics) -> Self {
        let params = &PxCookingParams::new(&**physics).expect("failed to create cooking params");
        let cooking = PxCooking::new(physics.foundation_mut(), &params).expect("failed to create cooking");
        Self(cooking)
    }
}

#[derive(Resource, Default)]
pub struct BPxTimeSync {
    timestep: f32,
    speed_factor: f32,
    bevy_physx_delta: f32,
}

impl BPxTimeSync {
    pub fn new(timestep: f32) -> Self {
        Self { timestep, speed_factor: 1., ..default() }
    }

    pub fn get_delta(&self) -> f32 {
        self.bevy_physx_delta
    }

    pub fn advance_bevy_time(&mut self, time: &Time) {
        self.bevy_physx_delta += time.delta_seconds() * self.speed_factor;
    }

    pub fn check_advance_physx_time(&mut self) -> Option<f32> {
        if self.bevy_physx_delta >= self.timestep {
            self.bevy_physx_delta -= self.timestep;
            Some(self.timestep)
        } else {
            None
        }
    }
}

#[derive(Resource, Deref, DerefMut, Default)]
pub struct BPxDefaultMaterial(Option<Handle<BPxMaterial>>);
