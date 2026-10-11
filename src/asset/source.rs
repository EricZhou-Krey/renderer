use crate::{
    asset::{MeshAsset, TextureAsset},
    component::{camera::CameraProjection, Transform},
};

#[derive(Debug, Clone, Default)]
pub struct SceneSource {
    pub lights: Vec<LightSource>,
    pub meshes: Vec<MeshSource>,
    pub textures: Vec<TextureSource>,
    pub materials: Vec<MaterialSource>,
    pub objects: Vec<ObjectSource>,
    pub camera: Option<CameraSource>,
}

#[derive(Debug, Clone)]
pub enum MeshSource {
    Description(MeshAsset),
}

#[derive(Debug, Clone)]
pub enum TextureSource {
    Description(TextureAsset),
}

#[derive(Debug, Clone)]
pub enum MaterialSource {
    Description {
        base_color: [f32; 4],
        base_color_texture: usize,
        normal_texture: usize,
    },
}

#[derive(Debug, Clone)]
pub struct ObjectSource {
    pub transform: Transform,
    pub mesh: usize,
    pub material: usize,
}

#[derive(Debug, Clone)]
pub struct LightSource {
    pub transform: Transform,
    pub color: [f32; 3],
    pub intensity: f32,
    pub range: f32,
}

#[derive(Debug, Clone)]
pub struct CameraSource {
    pub transform: Transform,
    pub projection: CameraProjection,
}
