use crate::asset::id::{MaterialId, MeshId, TextureId};

pub mod camera;
pub mod debug;

#[derive(Debug, Clone, bevy_ecs::component::Component, PartialEq)]
pub struct Mesh {
    pub id: MeshId,
}

#[derive(Debug, Clone, bevy_ecs::component::Component, PartialEq)]
pub struct Texture {
    pub id: TextureId,
}

#[derive(Debug, Clone, bevy_ecs::component::Component, PartialEq)]
pub struct Material {
    pub id: MaterialId,
}

#[derive(Default, bevy_ecs::component::Component, Debug, Clone, PartialEq)]
pub struct Transform {
    pub scale: glam::Vec3,
    pub translation: glam::Vec3,
    pub rotation: glam::Quat,
}

impl Transform {
    pub fn to_raw(&self) -> [[f32; 4]; 4] {
        glam::Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
            .to_cols_array_2d()
    }
    pub fn to_inverse_raw(&self) -> [[f32; 4]; 4] {
        glam::Mat4::from_scale_rotation_translation(self.scale, self.rotation, self.translation)
            .inverse()
            .to_cols_array_2d()
    }
}

#[derive(Default, bevy_ecs::component::Component, Debug, Clone, PartialEq)]
pub struct Light {
    pub color: [f32; 3],
    pub intensity: f32,
    pub range: f32,
}
