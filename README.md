<p align="left">
  <img src="https://user-images.githubusercontent.com/999113/253824185-ade6f3c1-0ce7-4e95-833a-daa619acbcb6.png" width="48%">
&nbsp;
  <img src="https://user-images.githubusercontent.com/999113/253824183-11d21bb3-700d-4a0b-aab4-60b48af49c23.png" width="48%">
</p>

### Bevy plugin for PhysX 5

[PhysX](https://github.com/NVIDIA-Omniverse/PhysX) is an open-source Physics SDK written in C++ and developed by Nvidia. \
This crate is a bridge between Bevy ECS and Rust [bindings](https://github.com/EmbarkStudios/physx-rs) made by Embark Studios.

[<img alt="github" src="https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github" height="20">](https://github.com/rlidwka/bevy_mod_physx)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs-8da0cb?style=for-the-badge&labelColor=555555&logo=docs.rs" height="20">](https://docs.rs/bevy_mod_physx)
[<img alt="crates.io" src="https://img.shields.io/crates/v/bevy_mod_physx.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/bevy_mod_physx)

### Getting started

Here is a snippet, which creates a ball bouncing on a fixed ground.

```rust
// this is similar to basic simulation example in rapier3d
use bevy::prelude::*;
use bevy_mod_physx::prelude::*;
use bevy_mod_physx::prelude::{Material, Shape}; // bevy prelude conflicts

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

|[]() |[]() |[]() |[]()
|--|--|--|--
| published | [bevy_mod_physx](https://crates.io/crates/bevy_mod_physx) v0.2.0 | Bevy 0.11 | PhysX 5
| | | |
| git tags | [git:master](https://github.com/rlidwka/bevy_mod_physx) | Bevy 0.11 | PhysX 5
| | [git:a21b570](https://github.com/rlidwka/bevy_mod_physx/tree/a21b570b20a1e7ac22b5c86c54fcc1597760f2ec) | Bevy 0.11 | PhysX 4
| | [git:43ae89e](https://github.com/rlidwka/bevy_mod_physx/tree/43ae89e013daf00ef841611149420fb4d04c2a4f) | Bevy 0.10 | PhysX 5
| | [git:8f66a99](https://github.com/rlidwka/bevy_mod_physx/tree/8f66a9965eb461794856898ca44a1faf13c295ab) | Bevy 0.10 | PhysX 4
| | | |
| other crates | [bevy_mod_physx](https://github.com/MasterOfMarkets/bevy_mod_physx) v0.1.0 | Bevy 0.10 | deprecated
| | [bevy_prototype_physx](https://github.com/superdump/bevy_prototype_physx) | Bevy 0.2-0.5 | unknown
| | [bevy_physx](https://crates.io/crates/bevy_physx) | | never existed
| | | |

*Note: you can find PhysX 4 version of this crate. It exists because PhysX 5 bindings don't have Vehicle API. It is not officially supported nor published to crates.io, and may get removed in the future.*
