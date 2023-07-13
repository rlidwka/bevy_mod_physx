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

pub struct PhysXDebugRenderPlugin;

#[derive(Resource, Default, Debug, Clone, Copy, Reflect)]
#[reflect(Resource)]
pub struct DebugRenderSettings {
    pub scale: f32,
    pub world_axes: f32,
    pub body_axes: f32,
    pub body_mass_axes: f32,
    pub body_lin_velocity: f32,
    pub body_ang_velocity: f32,
    pub contact_point: f32,
    pub contact_normal: f32,
    pub contact_error: f32,
    pub contact_force: f32,
    pub actor_axes: f32,
    pub collision_aabbs: f32,
    pub collision_shapes: f32,
    pub collision_axes: f32,
    pub collision_compounds: f32,
    pub collision_fnormals: f32,
    pub collision_edges: f32,
    pub collision_static: f32,
    pub collision_dynamic: f32,
    pub joint_local_frames: f32,
    pub joint_limits: f32,
    pub cull_box: f32,
    pub mbp_regions: f32,
    pub simulation_mesh: f32,
    pub sdf: f32,
}

impl Plugin for PhysXDebugRenderPlugin {
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
    /*let points = unsafe {
        std::slice::from_raw_parts(
            PxRenderBuffer_getPoints(buffer),
            PxRenderBuffer_getNbPoints(buffer) as usize,
        )
    };

    for point in points {
        dbg!(point.pos.to_bevy());
    }*/

    // display lines
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

    // display triangles
    /*let triangles = unsafe {
        std::slice::from_raw_parts(
            PxRenderBuffer_getTriangles(buffer),
            PxRenderBuffer_getNbTriangles(buffer) as usize,
        )
    };

    for triangle in triangles {
        dbg!(triangle.pos0.to_bevy());
    }*/
}
