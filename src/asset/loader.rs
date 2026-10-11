use crate::{
    asset::{
        id::{AssetId, MaterialId, MeshId, TextureId},
        manager::{MaterialManager, MeshManager, TextureManager},
        source::{MaterialSource, MeshSource, SceneSource, TextureSource},
        MaterialAsset,
    },
    component::{
        camera::{ActiveCamera, Camera, CameraProjection},
        Light, Material, Mesh, Transform,
    },
};
use bevy_ecs::{
    entity::Entity,
    event::Event,
    observer::On,
    prelude::{Commands, ResMut},
    resource::Resource,
};
use glam::Vec3;

#[derive(Default, Debug, Resource)]
pub struct GPUAssetUploader {
    pub asset_ids: Vec<AssetId>,
}

#[derive(Debug, Event)]
pub struct RequestLoadScene {
    pub scene: SceneSource,
}

pub fn load_scene(
    request: On<RequestLoadScene>,
    mut commands: Commands,
    mut mesh_manager: ResMut<MeshManager>,
    mut texture_manager: ResMut<TextureManager>,
    mut material_manager: ResMut<MaterialManager>,
    mut asset_uploader: ResMut<GPUAssetUploader>,
) {
    let scene: &SceneSource = &request.event().scene;

    if let Err(error) = validate_scene(scene) {
        commands.trigger(rook_terminal::event::TerminalOutput {
            content: error.to_string(),
            level: rook_terminal::event::OutputLevel::Error,
        });
        return;
    }

    let mut upload_ids: Vec<AssetId> = Vec::new();

    let mut texture_ids: Vec<TextureId> = Vec::with_capacity(scene.textures.len());

    for source in &scene.textures {
        let id: TextureId = match source {
            TextureSource::Description(asset) => texture_manager.textures.insert(asset.clone()),
            TextureSource::Source(id) => *id,
        };

        upload_ids.push(AssetId::Texture(id));
        texture_ids.push(id);
    }

    let mut mesh_ids: Vec<MeshId> = Vec::with_capacity(scene.meshes.len());

    for source in &scene.meshes {
        let id: MeshId = match source {
            MeshSource::Description(asset) => mesh_manager.meshes.insert(asset.clone()),
            MeshSource::Source(id) => *id,
        };

        upload_ids.push(AssetId::Mesh(id));
        mesh_ids.push(id);
    }

    let mut material_ids: Vec<MaterialId> = Vec::with_capacity(scene.materials.len());

    for source in &scene.materials {
        let id: MaterialId = match source {
            MaterialSource::Description {
                base_color,
                base_color_texture,
                normal_texture,
            } => material_manager.materials.insert(MaterialAsset {
                base_color: *base_color,
                base_color_texture: texture_ids[*base_color_texture],
                normal_texture: texture_ids[*normal_texture],
            }),
            MaterialSource::Source(id) => *id,
        };

        upload_ids.push(AssetId::Material(id));
        material_ids.push(id);
    }

    for object in &scene.objects {
        commands.spawn((
            object.transform.clone(),
            Mesh {
                id: mesh_ids[object.mesh],
            },
            Material {
                id: material_ids[object.material],
            },
        ));
    }

    for light_source in &scene.lights {
        commands.spawn((
            Light {
                color: light_source.color,
                intensity: light_source.intensity,
                range: light_source.range,
            },
            light_source.transform.clone(),
        ));
    }

    if let Some(scene_camera) = &scene.camera {
        let camera: Entity = commands
            .spawn((scene_camera.transform.clone(), Camera))
            .id();

        commands.insert_resource(scene_camera.projection.clone());
        commands.insert_resource(ActiveCamera { camera });
    } else {
        let camera: Entity = commands
            .spawn((
                Camera,
                Transform {
                    translation: Vec3::ZERO,
                    rotation: glam::Quat::IDENTITY,
                    scale: Vec3::ONE,
                },
            ))
            .id();

        commands.insert_resource(ActiveCamera { camera });
        commands.insert_resource(CameraProjection {
            vertical_fov: 60.0_f32.to_radians(),
            aspect_ratio: 16.0 / 9.0,
            z_near: 0.1,
            z_far: 1000.0,
        });
    }

    asset_uploader.asset_ids.append(&mut upload_ids);
}

fn validate_scene(scene: &SceneSource) -> anyhow::Result<()> {
    for (index, source) in scene.meshes.iter().enumerate() {
        let MeshSource::Description(mesh) = source else {
            continue;
        };

        for &vertex_index in &mesh.indices {
            anyhow::ensure!(
                (vertex_index as usize) < mesh.vertices.len(),
                "mesh {index} has out-of-range vertex index {vertex_index}"
            );
        }
    }

    for (index, source) in scene.textures.iter().enumerate() {
        let TextureSource::Description(texture) = source else {
            continue;
        };

        let expected_len: usize = (texture.width as usize)
            .checked_mul(texture.height as usize)
            .and_then(|pixel_count| pixel_count.checked_mul(4))
            .ok_or_else(|| anyhow::anyhow!("texture {index} dimensions overflow"))?;

        anyhow::ensure!(
            texture.rgba8.len() == expected_len,
            "texture {index}: expected {expected_len} RGBA8 bytes, got {}",
            texture.rgba8.len()
        );
    }

    for (index, source) in scene.materials.iter().enumerate() {
        let MaterialSource::Description {
            base_color_texture,
            normal_texture,
            ..
        } = source
        else {
            continue;
        };

        for texture_index in [*base_color_texture, *normal_texture] {
            anyhow::ensure!(
                texture_index < scene.textures.len(),
                "material {index} references missing texture {texture_index}"
            );
        }
    }

    for (index, object) in scene.objects.iter().enumerate() {
        anyhow::ensure!(
            object.mesh < scene.meshes.len(),
            "object {index} references missing mesh {}",
            object.mesh
        );
        anyhow::ensure!(
            object.material < scene.materials.len(),
            "object {index} references missing material {}",
            object.material
        );
    }

    Ok(())
}
