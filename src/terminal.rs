use crate::{
    commands::{
        AddLightCommand, AddLineCommand, AddObjectCommand, AddPointCommand, AnimateObjectCommand,
        EditObjectCommand,
    },
    component::{Light, Material, Mesh, Transform},
};
use bevy_ecs::world::World;
use rook_terminal::{
    file_system::{CommandRegistry, VFSChildren, VFSFilterExtension, VFSName, VFSQueryChildren},
    TerminalSession,
};

pub trait RendererTerminalWorldExtension {
    fn setup_render_terminal(&mut self);
}

impl RendererTerminalWorldExtension for World {
    fn setup_render_terminal(&mut self) {
        self.register_component::<VFSName>();
        self.register_component::<Transform>();
        self.register_component::<Mesh>();
        self.register_component::<Material>();
        self.register_component::<Light>();

        self.resource_scope(
            |world: &mut World, terminal_session: bevy_ecs::world::Mut<'_, TerminalSession>| {
                let object_filter = world.filter::<VFSName>()
                    & world.filter::<Transform>()
                    & world.filter::<Mesh>()
                    & world.filter::<Material>();

                let light_filter = world.filter::<VFSName>()
                    & world.filter::<Transform>()
                    & world.filter::<Light>();

                let objects_directory = world
                    .spawn((
                        VFSName {
                            name: "objects".to_string(),
                        },
                        VFSChildren {
                            children: Vec::new(),
                        },
                        VFSQueryChildren {
                            filter: object_filter,
                        },
                    ))
                    .id();

                let lights_directory = world
                    .spawn((
                        VFSName {
                            name: "lights".to_string(),
                        },
                        VFSChildren {
                            children: Vec::new(),
                        },
                        VFSQueryChildren {
                            filter: light_filter,
                        },
                    ))
                    .id();

                world
                    .entity_mut(terminal_session.root_entity)
                    .modify_component(|vfs_children: &mut VFSChildren| {
                        vfs_children.children.push(objects_directory);
                        vfs_children.children.push(lights_directory);
                    });
            },
        );

        self.resource_scope(
            |world: &mut World, mut command_registry: bevy_ecs::world::Mut<'_, CommandRegistry>| {
                command_registry.register(
                    AddObjectCommand::name(),
                    world.register_system(AddObjectCommand::execute),
                );
                command_registry.register(
                    EditObjectCommand::name(),
                    world.register_system(EditObjectCommand::execute),
                );
                command_registry.register(
                    AnimateObjectCommand::name(),
                    world.register_system(AnimateObjectCommand::execute),
                );
                command_registry.register(
                    AddLightCommand::name(),
                    world.register_system(AddLightCommand::execute),
                );
                command_registry.register(
                    AddPointCommand::name(),
                    world.register_system(AddPointCommand::execute),
                );
                command_registry.register(
                    AddLineCommand::name(),
                    world.register_system(AddLineCommand::execute),
                );
            },
        );
    }
}
/*

Examples:

add_object --name cube --mesh cube --material white_matte --position 0,0,0 --scale 1,1,1
edit_object --object cube --material black_matte --position 2,0,0
animate_object --object cube --transform 0,2,0 --duration 2
add_light --name key_light --color 1,0.9,0.8 --range 10 --intensity 5 --xyz 2,4,1
add_point --name marker --xyz 1,0,0 --size 0.1
add_line --name segment --start 0,0,0 --end 1,2,0 --width 0.025
*/
