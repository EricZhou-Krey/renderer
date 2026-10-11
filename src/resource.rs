use bevy_ecs::{resource::Resource, world::World};

#[derive(Default, Debug, PartialEq, Resource)]
pub struct FrameDelta(pub f32);

#[derive(Debug, Resource)]
pub struct TerminalDebugSettings {
    pub active: bool,
}

impl Default for TerminalDebugSettings {
    fn default() -> Self {
        Self { active: true }
    }
}

pub trait ResoureWorldExtension {
    fn setup_resource(&mut self);
}

impl ResoureWorldExtension for World {
    fn setup_resource(&mut self) {
        self.insert_resource(TerminalDebugSettings::default());
        self.insert_resource(FrameDelta::default());
    }
}
