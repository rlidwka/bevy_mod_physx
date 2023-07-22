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

### Compatibility / Prior art

 - published crates
   - [bevy_mod_physx](https://crates.io/crates/bevy_mod_physx) v0.2.0 - Bevy 0.11, PhysX 5

 - git tags
   - [git:master](https://github.com/rlidwka/bevy_physx) - Bevy 0.11, PhysX 5
   - [git:43ae89](https://github.com/rlidwka/bevy_physx/tree/43ae89e013daf00ef841611149420fb4d04c2a4f) - Bevy 0.10, PhysX 5
   - [git:467a45](https://github.com/rlidwka/bevy_physx/tree/467a452eb94b069a6c997eadb0dcd13211e44673) - Bevy 0.11, PhysX 4
   - [git:8f66a9](https://github.com/rlidwka/bevy_physx/tree/8f66a9965eb461794856898ca44a1faf13c295ab) - Bevy 0.10, PhysX 4

 - other crates
   - [bevy_mod_physx](https://github.com/MasterOfMarkets/bevy_mod_physx) - Bevy 0.10 (deprecated)
   - [bevy_prototype_physx](https://github.com/superdump/bevy_prototype_physx) - Bevy 0.2-0.5 (unknown)
   - [bevy_physx](https://crates.io/crates/bevy_physx) - (never existed)
