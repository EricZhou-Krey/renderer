use bevy_ecs::world::World;

pub trait RendererTerminalWorldExtension {
    fn setup_render_commands(&mut self);
}

impl RendererTerminalWorldExtension for World {
    fn setup_render_commands(&mut self) {}
}
