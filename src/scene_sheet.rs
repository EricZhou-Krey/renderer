use crate::{
    asset::{
        source::{
            CameraSource, LightSource, MaterialSource, MeshSource, ObjectSource, SceneSource,
            TextureSource,
        },
        MeshAsset, TextureAsset, Vertex,
    },
    component::{camera::CameraProjection, Transform},
};

pub fn demo1_scene_source() -> SceneSource {
    let camera_position = glam::Vec3::new(5.0, -5.0, 5.0);
    let direction = (glam::Vec3::ZERO - camera_position).normalize();

    SceneSource {
        camera: Some(CameraSource {
            transform: Transform {
                translation: camera_position,
                rotation: glam::Quat::from_rotation_arc(glam::Vec3::NEG_Z, direction),
                scale: glam::Vec3::ONE,
            },
            projection: CameraProjection {
                vertical_fov: 60.0_f32.to_radians(),
                aspect_ratio: 16.0 / 9.0,
                z_near: 0.1,
                z_far: 1000.0,
            },
        }),
        lights: vec![
            LightSource {
                transform: Transform {
                    translation: glam::Vec3::new(5.0, 0.0, 0.0),
                    rotation: glam::Quat::IDENTITY,
                    scale: glam::Vec3::ONE,
                },
                color: [100.0, 50.0, 50.0],
                intensity: 0.01,
                range: 1000.0,
            },
            LightSource {
                transform: Transform {
                    translation: glam::Vec3::new(0.0, 5.0, 0.0),
                    rotation: glam::Quat::IDENTITY,
                    scale: glam::Vec3::ONE,
                },
                color: [50.0, 100.0, 50.0],
                intensity: 0.01,
                range: 1000.0,
            },
            LightSource {
                transform: Transform {
                    translation: glam::Vec3::new(0.0, 0.0, 5.0),
                    rotation: glam::Quat::IDENTITY,
                    scale: glam::Vec3::ONE,
                },
                color: [50.0, 50.0, 100.0],
                intensity: 0.01,
                range: 1000.0,
            },
        ],
        meshes: vec![MeshSource::Description(demo_sphere_mesh())],
        textures: vec![
            TextureSource::Description(TextureAsset {
                width: 2,
                height: 2,
                rgba8: vec![
                    230, 90, 30, 255, 245, 210, 120, 255, 245, 210, 120, 255, 230, 90, 30, 255,
                ],
            }),
            TextureSource::Description(TextureAsset {
                width: 1,
                height: 1,
                rgba8: vec![128, 128, 255, 255],
            }),
        ],
        materials: vec![MaterialSource::Description {
            base_color: [1.0, 1.0, 1.0, 1.0],
            base_color_texture: 0,
            normal_texture: 1,
        }],
        objects: vec![ObjectSource {
            transform: Transform {
                translation: glam::Vec3::ZERO,
                rotation: glam::Quat::IDENTITY,
                scale: glam::Vec3::splat(1.5),
            },
            mesh: 0,
            material: 0,
        }],
    }
}

fn demo_sphere_mesh() -> MeshAsset {
    const SEGMENTS: u32 = 32;
    const RINGS: u32 = 16;

    let stride = SEGMENTS + 1;
    let mut vertices = Vec::with_capacity(((RINGS + 1) * stride) as usize);
    let mut indices = Vec::with_capacity((RINGS * SEGMENTS * 6) as usize);

    for ring in 0..=RINGS {
        let v = ring as f32 / RINGS as f32;
        let theta = v * std::f32::consts::PI;
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();

        for segment in 0..=SEGMENTS {
            let u = segment as f32 / SEGMENTS as f32;
            let phi = u * std::f32::consts::TAU;

            let position = [sin_theta * phi.cos(), cos_theta, sin_theta * phi.sin()];

            vertices.push(Vertex {
                position,
                normal: position,
                uv: [u, 1.0 - v],
            });
        }
    }

    for ring in 0..RINGS {
        for segment in 0..SEGMENTS {
            let a = ring * stride + segment;
            let b = a + stride;
            let c = a + 1;
            let d = b + 1;

            // Outward-facing triangle winding.
            indices.extend_from_slice(&[a, c, b, c, d, b]);
        }
    }

    MeshAsset { vertices, indices }
}
