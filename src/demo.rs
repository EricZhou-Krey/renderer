use bevy_ecs::world::World;

use crate::{
    asset::builtin::BuiltinAssets,
    component::{Light, Material, Mesh, Transform},
};

use rook_terminal::file_system::VFSName;

pub fn setup_demo_scene(world: &mut World) {
    world.resource_scope(
        |world: &mut World, builtins: bevy_ecs::world::Mut<'_, BuiltinAssets>| {
            let objects = [
                ("demo_square", builtins.meshes.square, -3.0),
                ("demo_cube", builtins.meshes.cube, -1.0),
                ("demo_sphere", builtins.meshes.sphere, 1.0),
                ("demo_cylinder", builtins.meshes.cylinder, 3.0),
            ];

            for (name, mesh_id, x) in objects {
                world.spawn((
                    VFSName {
                        name: name.to_string(),
                    },
                    Mesh { id: mesh_id },
                    Material {
                        id: builtins.materials.white_matte,
                    },
                    Transform {
                        translation: glam::Vec3::new(x, 0.75, 0.0),
                        scale: glam::Vec3::splat(1.5),
                        rotation: glam::Quat::IDENTITY,
                    },
                ));
            }

            world.spawn((
                VFSName {
                    name: "demo_light".to_string(),
                },
                Light {
                    color: [1.0, 1.0, 1.0],
                    intensity: 4.0,
                    range: 12.0,
                },
                Transform {
                    translation: glam::Vec3::new(0.0, 4.0, 3.0),
                    scale: glam::Vec3::ONE,
                    rotation: glam::Quat::IDENTITY,
                },
            ));
        },
    );
}
