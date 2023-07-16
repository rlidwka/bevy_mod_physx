<p align="center">
  <img src="https://user-images.githubusercontent.com/999113/253824185-ade6f3c1-0ce7-4e95-833a-daa619acbcb6.png" width="48%">
&nbsp;
  <img src="https://user-images.githubusercontent.com/999113/253824183-11d21bb3-700d-4a0b-aab4-60b48af49c23.png" width="48%">
</p>

### Getting started

Here is a snippet, which creates a ball bouncing on a fixed ground.

```rust
// this is similar to basic simulation example in rapier3d
use bevy::prelude::*;
use bevy_physx::prelude::*;
use bevy_physx::prelude::{Material, Shape}; // bevy prelude conflicts

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PhysicsPlugins)
        .add_systems(Startup, setup_graphics)
        .add_systems(Startup, setup_physics)
        .insert_resource(DebugRenderSettings::enable())
        .run();
}

fn setup_graphics(mut commands: Commands) {
    // Add a camera so we can see the debug-render.
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-3.0, 3.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}

fn setup_physics(
    mut commands: Commands,
    mut physics: ResMut<Physics>,
    mut geometries: ResMut<Assets<Geometry>>,
    mut materials: ResMut<Assets<Material>>,
) {
    // Create the ground.
    commands.spawn((
        RigidBody::Static,
        Shape {
            geometry: geometries.add(Geometry::halfspace(Vec3::Y)),
            material: materials.add(Material::new(&mut physics, 0.5, 0.5, 0.6)),
            ..default()
        },
        SpatialBundle::from_transform(Transform::from_xyz(0.0, -2.0, 0.0)),
    ));

    // Create the bouncing ball.
    commands.spawn((
        RigidBody::Dynamic,
        Shape {
            geometry: geometries.add(Geometry::ball(0.5)),
            material: materials.add(Material::new(&mut physics, 0.5, 0.5, 0.6)),
            ..default()
        },
        SpatialBundle::from_transform(Transform::from_xyz(0.0, 4.0, 0.0)),
    ));
}
```
