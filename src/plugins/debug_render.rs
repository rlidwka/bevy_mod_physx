//! Draw debug visualization on the screen.
//!
//! In order to enable debug visualization, you must insert
//! [DebugRenderSettings] resource. `scale` parameter has to be set
//! to a positive value, the rest of the parameters are set depending
//! on what you want to visualize.
//!
//! Example:
//! ```no_run
//! app.insert_resource(DebugRenderSettings {
//!     scale: 1.,
//!     collision_shapes: 1.,
//!     ..default()
//! })
//! ```
//!
//! Note: some visualizations are currently not implemented due to
//!       lack of support of trimesh gizmos in bevy.
//!
use bevy::prelude::*;
use physx::traits::Class;
use physx_sys::{
    PxRenderBuffer_getLines,
    PxRenderBuffer_getNbLines,
    PxScene_getRenderBuffer_mut,
    PxScene_getVisualizationParameter,
    PxScene_setVisualizationParameter_mut,
    PxVisualizationParameter,
};

use crate::prelude::{self as bpx, *};

pub struct DebugRenderPlugin;

#[derive(Resource, Default, Debug, Clone, Copy, Reflect)]
#[reflect(Resource)]
/// Set debug visualization parameters.
pub struct DebugRenderSettings {
    /// This overall visualization scale gets multiplied with the individual scales.
    ///
    /// Setting to zero ignores all visualizations. Default is 0.
    pub scale: f32,
    /// Visualize the world axes.
    pub world_axes: f32,
    /// Visualize a bodies axes.
    pub body_axes: f32,
    /// Visualize a body’s mass axes.
    ///
    /// This visualization is also useful for visualizing the sleep state of bodies.
    /// Sleeping bodies are drawn in black, while awake bodies are drawn in white.
    /// If the body is sleeping and part of a sleeping group, it is drawn in red.
    pub body_mass_axes: f32,
    /// Visualize the bodies linear velocity.
    pub body_lin_velocity: f32,
    /// Visualize the bodies angular velocity.
    pub body_ang_velocity: f32,
    /// Visualize contact points.
    pub contact_point: f32,
    /// Visualize contact normals.
    pub contact_normal: f32,
    /// Visualize contact errors.
    pub contact_error: f32,
    /// Visualize contact forces.
    pub contact_force: f32,
    /// Visualize actor axes.
    pub actor_axes: f32,
    /// Visualize bounds (AABBs in world space)
    pub collision_aabbs: f32,
    /// Shape visualization.
    pub collision_shapes: f32,
    /// Shape axis visualization.
    pub collision_axes: f32,
    /// Compound visualization (compound AABBs in world space).
    pub collision_compounds: f32,
    /// Mesh & convex face normals.
    pub collision_fnormals: f32,
    /// Active edges for meshes.
    pub collision_edges: f32,
    /// Static pruning structures.
    pub collision_static: f32,
    /// Dynamic pruning structures.
    pub collision_dynamic: f32,
    /// Joint local axes.
    pub joint_local_frames: f32,
    /// Joint limits.
    pub joint_limits: f32,
    /// Visualize culling box.
    pub cull_box: f32,
    /// MBP regions.
    pub mbp_regions: f32,
    /// Renders the simulation mesh instead of the collision mesh
    /// (only available for tetmeshes).
    pub simulation_mesh: f32,
    /// Renders the SDF of a mesh instead of the collision mesh
    /// (only available for triangle meshes with SDFs).
    pub sdf: f32,
}

impl DebugRenderSettings {
    pub fn enable() -> Self {
        Self {
            scale: 1.,
            collision_shapes: 1.,
            ..default()
        }
    }
}

impl Plugin for DebugRenderPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<DebugRenderSettings>();
        app.add_systems(
            Update,
            (set_visualization_params, debug_visualization)
                .run_if(resource_exists::<DebugRenderSettings>()),
        );
    }
}

fn set_visualization_params(
    mut scene: ResMut<bpx::Scene>,
    mut vis_params: ResMut<DebugRenderSettings>,
) {
    if !vis_params.is_changed() { return; };

    let mut scene = scene.get_mut();

    macro_rules! set {
        ($key: ident, $param: ident) => {
            unsafe {
                if vis_params.$key >= 0. {
                    PxScene_setVisualizationParameter_mut(
                        scene.as_mut_ptr(),
                        PxVisualizationParameter::$param,
                        vis_params.$key,
                    );
                }

                vis_params.$key = PxScene_getVisualizationParameter(
                    scene.as_mut_ptr(),
                    PxVisualizationParameter::$param,
                );
            }
        };
    }

    set!(scale, Scale);
    set!(world_axes, WorldAxes);
    set!(body_axes, BodyAxes);
    set!(body_mass_axes, BodyMassAxes);
    set!(body_lin_velocity, BodyLinVelocity);
    set!(body_ang_velocity, BodyAngVelocity);
    set!(contact_point, ContactPoint);
    set!(contact_normal, ContactNormal);
    set!(contact_error, ContactError);
    set!(contact_force, ContactForce);
    set!(actor_axes, ActorAxes);
    set!(collision_aabbs, CollisionAabbs);
    set!(collision_shapes, CollisionShapes);
    set!(collision_axes, CollisionAxes);
    set!(collision_compounds, CollisionCompounds);
    set!(collision_fnormals, CollisionFnormals);
    set!(collision_edges, CollisionEdges);
    set!(collision_static, CollisionStatic);
    set!(collision_dynamic, CollisionDynamic);
    set!(joint_local_frames, JointLocalFrames);
    set!(joint_limits, JointLimits);
    set!(cull_box, CullBox);
    set!(mbp_regions, MbpRegions);
    set!(simulation_mesh, SimulationMesh);
    set!(sdf, Sdf);
}

fn debug_visualization(
    mut gizmos: Gizmos,
    mut scene: ResMut<bpx::Scene>,
) {
    let mut scene = scene.get_mut();
    let buffer = unsafe { PxScene_getRenderBuffer_mut(scene.as_mut_ptr()) };

    // display points
    /*let ptr = unsafe { PxRenderBuffer_getPoints(buffer) };
    if !ptr.is_null() {
        let points = unsafe {
            std::slice::from_raw_parts(
                PxRenderBuffer_getPoints(buffer),
                PxRenderBuffer_getNbPoints(buffer) as usize,
            )
        };

        for point in points {
            dbg!(point.pos.to_bevy());
        }
    }*/

    // display lines
    let ptr = unsafe { PxRenderBuffer_getLines(buffer) };
    if !ptr.is_null() {
        let lines = unsafe {
            std::slice::from_raw_parts(
                PxRenderBuffer_getLines(buffer),
                PxRenderBuffer_getNbLines(buffer) as usize,
            )
        };

        for line in lines {
            assert_eq!(line.color0, line.color1);
            let color: [u8; 4] = line.color0.to_ne_bytes();
            gizmos.line(
                line.pos0.to_bevy(),
                line.pos1.to_bevy(),
                Color::rgba_u8(color[0], color[1], color[2], color[3]),
            );
        }
    }

    // display triangles
    /*let ptr = unsafe { PxRenderBuffer_getTriangles(buffer) };
    if !ptr.is_null() {
        let triangles = unsafe {
            std::slice::from_raw_parts(
                PxRenderBuffer_getTriangles(buffer),
                PxRenderBuffer_getNbTriangles(buffer) as usize,
            )
        };

        for triangle in triangles {
            dbg!(triangle.pos0.to_bevy());
        }
    }*/
}
