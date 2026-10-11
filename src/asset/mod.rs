use crate::asset::{
    builtin::install_builtin_assets_system,
    id::TextureId,
    loader::{load_scene, GPUAssetUploader},
    manager::{MaterialManager, MeshManager, TextureManager},
};
use bevy_ecs::{system::RunSystemOnce, world::World};

pub mod builtin;
pub mod id;
pub mod loader;
pub mod manager;
pub mod source;

#[repr(C)]
#[derive(Default, Debug, PartialEq, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct MeshAsset {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct TextureAsset {
    pub width: u32,
    pub height: u32,
    pub rgba8: Vec<u8>,
}

#[derive(Default, Debug, PartialEq, Clone)]
pub struct MaterialAsset {
    pub base_color: [f32; 4],
    pub base_color_texture: TextureId,
    pub normal_texture: TextureId,
}

pub trait AssetWorldExtension {
    fn setup_assets(&mut self);
}

impl AssetWorldExtension for World {
    fn setup_assets(&mut self) {
        self.insert_resource(MeshManager::default());
        self.insert_resource(TextureManager::default());
        self.insert_resource(MaterialManager::default());
        self.insert_resource(GPUAssetUploader::default());

        self.run_system_once(install_builtin_assets_system)
            .expect("failed to install built-in assets");
        self.add_observer(load_scene);
    }
}
