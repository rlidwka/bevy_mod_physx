//! Abstract singleton factory class used for instancing objects in the Physics SDK.
use bevy::prelude::*;
use derive_more::{Deref, DerefMut};
use physx::prelude::*;
use physx_sys::PxTolerancesScale;

use crate::types::*;

struct ErrorCallback;

impl physx::physics::ErrorCallback for ErrorCallback {
    fn report_error(&self, _code: enumflags2::BitFlags<physx::foundation::ErrorCode>, message: &str, file: &str, line: u32) {
        bevy::log::error!(target: "bevy_mod_physx", "[{file:}:{line:}]: {message:}");
    }
}

#[derive(Resource, Deref, DerefMut)]
pub struct Physics(PhysicsFoundation<physx::foundation::DefaultAllocator, PxShape>);

impl Physics {
    pub fn new(foundation_desc: &FoundationDescriptor) -> Self {
        let mut builder = physx::physics::PhysicsFoundationBuilder::default();
        builder.enable_visual_debugger(foundation_desc.visual_debugger);
        builder.with_extensions(foundation_desc.extensions);
        builder.with_vehicle_sdk(true);
        builder.set_pvd_port(foundation_desc.visual_debugger_port);
        if let Some(host) = foundation_desc.visual_debugger_host.as_ref() {
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
        Self(physics)
    }
}

#[derive(Clone)]
/// Descriptor class for creating a physics foundation.
pub struct FoundationDescriptor {
    /// Initialize the PhysXExtensions library.
    ///
    /// Default: true
    pub extensions: bool,
    /// Values used to determine default tolerances for objects at creation time.
    ///
    /// Default: length=1, speed=10
    pub tolerances: PxTolerancesScale,
    /// Enable visual debugger (PVD).
    ///
    /// Default: false
    pub visual_debugger: bool,
    /// IP port used for PVD, should same as the port setting
    /// in PVD application.
    ///
    /// Default: 5425
    pub visual_debugger_port: i32,
    /// Host address of the PVD application.
    ///
    /// Default: localhost
    pub visual_debugger_host: Option<String>,
}

impl Default for FoundationDescriptor {
    fn default() -> Self {
        Self {
            extensions: true,
            tolerances: PxTolerancesScale { length: 1., speed: 10. },
            visual_debugger: false,
            visual_debugger_port: 5425,
            visual_debugger_host: None,
        }
    }
}
