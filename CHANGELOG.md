## 0.8.0 - bevy 0.16 - 2025-05-15

### Changed

 - updated Bevy to 0.16

## 0.7.0 - bevy 0.15 - 2024-12-16

### Changed

 - updated Bevy to 0.15
 - disabled bevy's default-features, enabling only features this crate needs

## 0.6.0 - bevy 0.14 - 2024-07-07

### Added

 - added functions to create geometry from bevy mesh, [#8](https://github.com/rlidwka/bevy_mod_physx/pull/8)

### Changed

 - updated Bevy to 0.14

## 0.5.0 - bevy 0.13 - 2024-03-03

### Changed

 - updated Bevy to 0.13

## 0.4.1 - bevy 0.12 - 2024-03-02

### Fixed

 - fixed null pointer handling in debug visualization (it now crashes as debug assertion in newer Rust versions)

## 0.4.0 - bevy 0.12 - 2023-11-17

### Changed

 - updated Bevy to 0.12

## 0.3.2 - bevy 0.11 - 2023-11-17

### Added

 - added `lock_flags` plugin (`RigidDynamicLockFlags`)

## 0.3.1 - bevy 0.11 - 2023-10-13

### Fixed

 - updated physx-rs to newer version, removed workarounds for now fixed issues there

## 0.3.0 - bevy 0.11 - 2023-08-07

### Added

 - added default material as an argument for `PhysicsCore` plugin
 - added articulation joint velocity and position sync

### Changed

 - crate structure has been reworked, so many structs are moved to different places
   - `bevy_mod_physx::SceneDescriptor` -> `bevy_mod_physx::core::scene::SceneDescriptor`
   - `bevy_mod_physx::components::Geometry` -> `bevy_mod_physx::core::geometry::Geometry`
   - `bevy_mod_physx::callbacks::OnAdvance` -> `bevy_mod_physx::types::OnAdvance`
   - `bevy_mod_physx::plugins::Velocity` -> `bevy_mod_physx::plugins::velocity::Velocity`
   - etc.
 - visual debugger is disabled by default, add plugin `PhysicsCore::new().with_pvd()` to enable
 - in articulations, `drive_xxx` is renamed to simply `xxx` (e.g. `drive_swing1` -> `swing1`)

### Fixed

 - changing Transform of an articulation root now correctly syncs to physx
 - fixed API for scaling an existing convex mesh

## 0.2.1 - bevy 0.11 - 2023-07-26

### Fixed

 - fix PhysX warning when using Velocity plugin with kinematic bodies

## 0.2.0 - bevy 0.11 - 2023-07-23

Initial release.

## 0.1.0 - bevy 0.10 - 2023-03-14

This is a different crate, which is now deprecated.

See [MasterOfMarkets/bevy_mod_physx](https://github.com/MasterOfMarkets/bevy_mod_physx) for details.
