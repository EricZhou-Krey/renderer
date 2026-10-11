use crate::{asset::FrameDelta, resource::terminal_debug_settings::TerminalDebugSettings};
use bevy_ecs::world::World;

pub mod acoustic_world;
pub mod asset_store;
pub mod audio_output;
pub mod simulation_settings;
pub mod terminal_debug_settings;

pub trait ResoureWorldExtension {
    fn setup_resource(&mut self);
}

impl ResoureWorldExtension for World {
    fn setup_resource(&mut self) {
        self.insert_resource(TerminalDebugSettings::default());
        self.insert_resource(FrameDelta::default());
    }
}
