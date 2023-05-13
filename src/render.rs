use crate::assets::GeometryInner;
use crate::prelude::*;
use bevy::prelude::*;
use bevy::reflect::TypeUuid;
use bevy::render::mesh::Indices;
use bevy::render::render_resource::{AsBindGroup, PrimitiveTopology, ShaderRef};
use physx::triangle_mesh::TriangleMeshIndices;
use std::collections::HashSet;

const SHADER_HANDLE: HandleUntyped = HandleUntyped::weak_from_u64(Shader::TYPE_UUID, 9326911668127598676);
const DEFAULT_COLOR: Color = Color::rgba(0.5, 0.7, 0.8, 1.);
pub struct PhysXDebugRenderPlugin;

#[derive(Resource, Debug, Clone, Copy, Reflect, FromReflect)]
#[reflect(Resource)]
pub struct DebugRenderSettings {
    pub visibility: Visibility,
    pub color: Color,
}

impl Default for DebugRenderSettings {
    fn default() -> Self {
        Self {
            visibility: Visibility::Hidden,
            color: DEFAULT_COLOR,
        }
    }
}

#[derive(Resource)]
pub struct DebugRenderMaterials {
    base: Handle<DebugRenderMaterial>,
}

impl Plugin for PhysXDebugRenderPlugin {
    fn build(&self, app: &mut App) {
        let mut shaders = app.world.get_resource_mut::<Assets<Shader>>().unwrap();
        shaders.set_untracked(SHADER_HANDLE, Shader::from_wgsl("
            #import bevy_pbr::mesh_view_bindings

            struct DebugRenderMaterial {
                color: vec4<f32>,
            };

            @group(1) @binding(0)
            var<uniform> material: DebugRenderMaterial;

            struct FragmentInput {
                #import bevy_pbr::mesh_vertex_output
            };

            struct FragmentOutput {
                @builtin(frag_depth) depth: f32,
                @location(0) color: vec4<f32>,
            };

            @fragment
            fn fragment(in: FragmentInput) -> FragmentOutput {
                var out: FragmentOutput;
                out.depth = 1.0;
                out.color = material.color;
                return out;
            }
        "));

        app.register_type::<DebugRenderSettings>();
        app.init_resource::<DebugRenderSettings>();
        app.add_plugin(MaterialPlugin::<DebugRenderMaterial>::default());

        let material = app.world.resource_mut::<Assets<DebugRenderMaterial>>().add(
            DebugRenderMaterial { color: DEFAULT_COLOR }
        );
        app.insert_resource(DebugRenderMaterials {
            base: material,
        });

        app.add_system(create_debug_meshes);
        app.add_system(toggle_debug_meshes_visibility);
    }
}

#[derive(Component)]
pub struct DebugRenderMesh;

#[derive(AsBindGroup, TypeUuid, Debug, Clone)]
#[uuid = "e87d45f2-b145-49c2-b457-1298556004e5"]
pub struct DebugRenderMaterial {
    #[uniform(0)]
    color: Color,
}

impl bevy::pbr::Material for DebugRenderMaterial {
    /*fn vertex_shader() -> ShaderRef {
        ShaderRef::Handle(SHADER_HANDLE.typed())
    }*/

    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(SHADER_HANDLE.typed())
    }
}

fn create_debug_meshes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    materials: Res<DebugRenderMaterials>,
    settings: Res<DebugRenderSettings>,
    mut geometries: ResMut<Assets<Geometry>>,
    query: Query<(Entity, &Shape), Added<Shape>>,
) {
    for (entity, shape) in query.iter() {
        let Some(geometry) = geometries.get_mut(&shape.geometry) else { continue; };
        let mut positions = vec![];
        let mut indices = vec![];
        const SPHERE_SEGMENTS: u32 = 24;

        match &geometry.obj {
            GeometryInner::Sphere(geom)  => {
                for i in 0..SPHERE_SEGMENTS {
                    let arclen = std::f32::consts::TAU / SPHERE_SEGMENTS as f32 * i as f32;
                    let (sin, cos) = arclen.sin_cos();
                    positions.push(Vec3::new(sin * geom.radius, cos * geom.radius, 0.));
                    positions.push(Vec3::new(sin * geom.radius, 0., cos * geom.radius));
                    positions.push(Vec3::new(0., sin * geom.radius, cos * geom.radius));

                    for j in 0..3 {
                        indices.push(i * 3 + j);
                        indices.push(((i + 1) % SPHERE_SEGMENTS) * 3 + j);
                    }
                }
            },
            GeometryInner::Plane { normal, .. } => {
                for x in 0..=0 {
                    indices.push(positions.len() as u32);
                    positions.push(Quat::from_rotation_arc(Vec3::X, *normal) * Vec3::new(0., x as f32, -1000000.));
                    indices.push(positions.len() as u32);
                    positions.push(Quat::from_rotation_arc(Vec3::X, *normal) * Vec3::new(0., x as f32, 1000000.));
                }

                for y in 0..=0 {
                    indices.push(positions.len() as u32);
                    positions.push(Quat::from_rotation_arc(Vec3::X, *normal) * Vec3::new(0., -1000000., y as f32));
                    indices.push(positions.len() as u32);
                    positions.push(Quat::from_rotation_arc(Vec3::X, *normal) * Vec3::new(0., 1000000., y as f32));
                }
            },
            GeometryInner::Capsule(geom)  => {
                for i in 0..SPHERE_SEGMENTS+2 {
                    let (arclen, offset) = if i <= SPHERE_SEGMENTS / 2 {
                        (std::f32::consts::TAU / SPHERE_SEGMENTS as f32 * i as f32, geom.halfHeight)
                    } else {
                        (std::f32::consts::TAU / SPHERE_SEGMENTS as f32 * (i - 1) as f32, -geom.halfHeight)
                    };
                    let (sin, cos) = arclen.sin_cos();
                    positions.push(Vec3::new(sin * geom.radius + offset, cos * geom.radius, 0.));
                    positions.push(Vec3::new(sin * geom.radius + offset, 0., cos * geom.radius));

                    for j in 0..2 {
                        indices.push(i * 2 + j);
                        indices.push(((i + 1) % (SPHERE_SEGMENTS + 2)) * 2 + j);
                    }
                }

                let pos_offset = positions.len() as u32;
                for i in 0..SPHERE_SEGMENTS {
                    let arclen = std::f32::consts::TAU / SPHERE_SEGMENTS as f32 * i as f32;
                    let (sin, cos) = arclen.sin_cos();
                    positions.push(Vec3::new(-geom.halfHeight, sin * geom.radius, cos * geom.radius));
                    positions.push(Vec3::new(geom.halfHeight, sin * geom.radius, cos * geom.radius));

                    for j in 0..2 {
                        indices.push(i * 2 + j + pos_offset);
                        indices.push(((i + 1) % SPHERE_SEGMENTS) * 2 + j + pos_offset);
                    }
                }
            },
            GeometryInner::Box(geom) => {
                let ext = geom.halfExtents;
                positions.push(Vec3::new(-ext.x, -ext.y, -ext.z));
                positions.push(Vec3::new(-ext.x, -ext.y, ext.z));
                positions.push(Vec3::new(-ext.x, ext.y, -ext.z));
                positions.push(Vec3::new(-ext.x, ext.y, ext.z));
                positions.push(Vec3::new(ext.x, -ext.y, -ext.z));
                positions.push(Vec3::new(ext.x, -ext.y, ext.z));
                positions.push(Vec3::new(ext.x, ext.y, -ext.z));
                positions.push(Vec3::new(ext.x, ext.y, ext.z));
                for idx in [0, 1, 0, 2, 1, 3, 2, 3, 4, 5, 4, 6, 5, 7, 6, 7, 0, 4, 1, 5, 2, 6, 3, 7] {
                    indices.push(idx);
                }
            },
            GeometryInner::ConvexMesh { mesh, scale, rotation, .. } => {
                let mesh = mesh.lock().unwrap();
                for vertex in mesh.get_vertices() {
                    positions.push(*rotation * vertex.to_bevy() * *scale);
                }

                let index_buffer = mesh.get_index_buffer();
                let mut dedup = HashSet::new();

                for idx in 0..mesh.get_nb_polygons() {
                    let polygon = mesh.get_polygon_data(idx).unwrap();
                    for i in polygon.index_base..polygon.index_base+polygon.nb_verts {
                        let next = if i + 1 == polygon.index_base+polygon.nb_verts { polygon.index_base } else { i + 1 };
                        let p1 = index_buffer[i as usize] as u32;
                        let p2 = index_buffer[next as usize] as u32;

                        if dedup.insert((p1.min(p2), p1.max(p2))) {
                            indices.push(p1);
                            indices.push(p2);
                        }
                    }
                }
            },
            GeometryInner::TriangleMesh { mesh, scale, rotation, .. } => {
                let mesh = mesh.lock().unwrap();
                for vertex in mesh.get_vertices() {
                    positions.push(*rotation * vertex.to_bevy() * *scale);
                }

                let index_buffer = mesh.get_triangles();
                let length = mesh.get_nb_triangles() * 3;
                let mut dedup = HashSet::new();

                for idx in (0..).step_by(3) {
                    if idx + 2 >= length { break; }

                    let idx = idx as usize;
                    let (point1, point2, point3) = match index_buffer {
                        TriangleMeshIndices::U16(vec) => (vec[idx] as u32, vec[idx+1] as u32, vec[idx+2] as u32),
                        TriangleMeshIndices::U32(vec) => (vec[idx], vec[idx+1], vec[idx+2]),
                    };

                    for (p1, p2) in [(point1, point2), (point2, point3), (point3, point1)] {
                        if dedup.insert((p1.min(p2), p1.max(p2))) {
                            indices.push(p1);
                            indices.push(p2);
                        }
                    }
                }
            },
            GeometryInner::HeightField { mesh, scale, .. } => {
                let mesh = mesh.lock().unwrap();
                let rows = mesh.get_nb_rows();
                let columns = mesh.get_nb_columns();
                let samples = mesh.save_cells();

                for row in 0..rows {
                    for column in 0..columns {
                        let sample = samples[(row * columns + column) as usize];
                        positions.push(*scale * Vec3::new(row as f32, sample.height as f32, column as f32));

                        if column != 0 {
                            indices.push(row * columns + column - 1);
                            indices.push(row * columns + column);
                        }

                        if row != 0 {
                            indices.push((row - 1) * columns + column);
                            indices.push(row * columns + column);
                        }
                    }
                }
            },
        }

        let mut mesh = Mesh::new(PrimitiveTopology::LineList);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.set_indices(Some(Indices::U32(indices)));

        let mesh_entity = commands.spawn(DebugRenderMesh)
            .insert(MaterialMeshBundle {
                mesh: meshes.add(mesh),
                material: materials.base.clone(),
                visibility: settings.visibility,
                ..default()
            })
            .id();

        commands.entity(entity).add_child(mesh_entity);
    }
}

fn toggle_debug_meshes_visibility(
    mut query: Query<&mut Visibility, With<DebugRenderMesh>>,
    settings: Res<DebugRenderSettings>,
    handles: Res<DebugRenderMaterials>,
    mut materials: ResMut<Assets<DebugRenderMaterial>>,
) {
    if !settings.is_changed() { return; }

    if let Some(base) = materials.get_mut(&handles.base) {
        base.color = settings.color;
    }

    for mut visibility in query.iter_mut() {
        *visibility = settings.visibility;
    }
}
