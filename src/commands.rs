use std::collections::HashMap;

use bevy_ecs::prelude::{Commands, In, Query, Res};
use glam::vec3;

use crate::{
    asset::{
        builtin::BuiltinAssets,
        id::{MaterialId, MeshId},
    },
    component::{Light, Material, Mesh, Transform},
    resource::FrameDelta,
};
use rook_terminal::{
    event::{OutputLevel, TerminalOutput},
    file_system::VFSName,
};

#[derive(Debug, bevy_ecs::component::Component)]
pub struct TransformAnimation {
    pub target: Transform,
    pub elapsed: f32,
    pub duration: f32,
}

fn unquote(value: &str) -> &str {
    value.trim().trim_matches(|c| c == '"' || c == '\'')
}

fn parse_options(args: &[String]) -> Result<HashMap<&str, &str>, String> {
    let mut options = HashMap::new();
    let mut i = 0;

    while i < args.len() {
        let Some(arg) = args[i].strip_prefix("--") else {
            return Err(format!("unexpected argument '{}'", args[i]));
        };

        if let Some((key, value)) = arg.split_once('=') {
            options.insert(key, unquote(value));
            i += 1;
        } else {
            let value = args
                .get(i + 1)
                .ok_or_else(|| format!("missing value for --{arg}"))?;

            if value.starts_with("--") {
                return Err(format!("missing value for --{arg}"));
            }

            options.insert(arg, unquote(value));
            i += 2;
        }
    }

    Ok(options)
}

fn parse_vec3(value: &str, option: &str) -> Result<glam::Vec3, String> {
    let values: Vec<f32> = value
        .split(',')
        .map(|part| {
            part.trim()
                .parse::<f32>()
                .map_err(|e| format!("invalid --{option} value '{value}': {e}"))
        })
        .collect::<Result<_, _>>()?;

    if values.len() != 3 {
        return Err(format!("--{option} must be x,y,z"));
    }

    let result = glam::Vec3::new(values[0], values[1], values[2]);
    if !result.is_finite() {
        return Err(format!("--{option} values must be finite"));
    }

    Ok(result)
}

fn parse_f32(value: &str, option: &str) -> Result<f32, String> {
    let value = value
        .parse::<f32>()
        .map_err(|e| format!("invalid --{option} value '{value}': {e}"))?;

    if !value.is_finite() {
        return Err(format!("--{option} must be finite"));
    }

    Ok(value)
}

fn parse_transform_options(
    options: &HashMap<&str, &str>,
    mut transform: Transform,
) -> Result<Transform, String> {
    if options.contains_key("position") && options.contains_key("transform") {
        return Err("use either --position or --transform, not both".to_string());
    }

    if let Some(value) = options.get("position").or_else(|| options.get("transform")) {
        let option = if options.contains_key("transform") {
            "transform"
        } else {
            "position"
        };
        transform.translation = parse_vec3(value, option)?;
    }

    if let Some(value) = options.get("scale") {
        transform.scale = parse_vec3(value, "scale")?;
    }

    if let Some(value) = options.get("rotation") {
        let euler_degrees = parse_vec3(value, "rotation")?;
        let euler_radians = vec3(
            euler_degrees.x.to_radians(),
            euler_degrees.y.to_radians(),
            euler_degrees.z.to_radians(),
        );
        transform.rotation = glam::Quat::from_euler(
            glam::EulerRot::XYZ,
            euler_radians.x,
            euler_radians.y,
            euler_radians.z,
        );
    }

    Ok(transform)
}

fn default_transform() -> Transform {
    Transform {
        translation: glam::Vec3::ZERO,
        scale: glam::Vec3::ONE,
        rotation: glam::Quat::IDENTITY,
    }
}

fn resolve_mesh(name: &str, builtins: &BuiltinAssets) -> Result<MeshId, String> {
    match name {
        "square" => Ok(builtins.meshes.square),
        "cube" => Ok(builtins.meshes.cube),
        "sphere" => Ok(builtins.meshes.sphere),
        "cylinder" => Ok(builtins.meshes.cylinder),
        _ => Err(format!(
            "unknown mesh '{name}'; choose square, cube, sphere, or cylinder"
        )),
    }
}

fn resolve_material(name: &str, builtins: &BuiltinAssets) -> Result<MaterialId, String> {
    match name {
        "white_matte" => Ok(builtins.materials.white_matte),
        "black_matte" => Ok(builtins.materials.black_matte),
        _ => Err(format!(
            "unknown material '{name}'; choose white_matte or black_matte"
        )),
    }
}

fn ensure_name_is_available(name: &str, existing_names: &Query<&VFSName>) -> Result<(), String> {
    if existing_names.iter().any(|existing| existing.name == name) {
        Err(format!("an entity named '{name}' already exists"))
    } else {
        Ok(())
    }
}

fn emit(commands: &mut Commands, content: impl Into<String>, level: OutputLevel) {
    commands.trigger(TerminalOutput {
        content: content.into(),
        level,
    });
}

pub struct AddObjectCommand;

impl AddObjectCommand {
    pub fn name() -> &'static str {
        "add_object"
    }

    pub fn execute(
        arguments: In<Vec<String>>,
        builtins: Res<BuiltinAssets>,
        existing_names: Query<&VFSName>,
        mut commands: Commands,
    ) {
        let result = (|| -> Result<_, String> {
            let options = parse_options(&arguments.0)?;

            let name = options
                .get("name")
                .ok_or_else(|| "missing --name".to_string())?
                .to_string();
            ensure_name_is_available(&name, &existing_names)?;

            let mesh_name = options.get("mesh").copied().unwrap_or("cube");
            let mesh_id = resolve_mesh(mesh_name, &builtins)?;

            let material_name = options.get("material").copied().unwrap_or("white_matte");
            let material_id = resolve_material(material_name, &builtins)?;

            let transform = parse_transform_options(&options, default_transform())?;

            Ok((
                name,
                mesh_name.to_string(),
                material_name.to_string(),
                mesh_id,
                material_id,
                transform,
            ))
        })();

        match result {
            Ok((name, mesh_name, material_name, mesh_id, material_id, transform)) => {
                commands.spawn((
                    VFSName { name: name.clone() },
                    Mesh { id: mesh_id },
                    Material { id: material_id },
                    transform,
                ));

                emit(
                    &mut commands,
                    format!("Created '{name}' using {mesh_name} and {material_name}"),
                    OutputLevel::Info,
                );
            }
            Err(error) => emit(
                &mut commands,
                format!("add_object: {error}"),
                OutputLevel::Error,
            ),
        }
    }
}

pub struct EditObjectCommand;

impl EditObjectCommand {
    pub fn name() -> &'static str {
        "edit_object"
    }

    pub fn execute(
        arguments: In<Vec<String>>,
        builtins: Res<BuiltinAssets>,
        mut objects: Query<(
            &VFSName,
            Option<&mut Mesh>,
            Option<&mut Material>,
            Option<&mut Transform>,
        )>,
        mut commands: Commands,
    ) {
        let result = (|| -> Result<(), String> {
            let options = parse_options(&arguments.0)?;

            let name = options
                .get("object")
                .ok_or_else(|| "missing --object".to_string())?;

            let mesh_id = options
                .get("mesh")
                .map(|name| resolve_mesh(name, &builtins))
                .transpose()?;

            let material_id = options
                .get("material")
                .map(|name| resolve_material(name, &builtins))
                .transpose()?;

            let has_transform_options = ["position", "transform", "scale", "rotation"]
                .iter()
                .any(|key| options.contains_key(key));

            if mesh_id.is_none() && material_id.is_none() && !has_transform_options {
                return Err("no changes specified".to_string());
            }

            let mut found = false;

            for (vfs_name, mut mesh, mut material, mut transform) in &mut objects {
                if vfs_name.name != *name {
                    continue;
                }

                if mesh_id.is_some() && mesh.is_none() {
                    return Err(format!("'{name}' has no mesh to edit"));
                }
                if material_id.is_some() && material.is_none() {
                    return Err(format!("'{name}' has no material to edit"));
                }
                if has_transform_options && transform.is_none() {
                    return Err(format!("'{name}' has no transform to edit"));
                }

                if let (Some(id), Some(component)) = (mesh_id, mesh.as_deref_mut()) {
                    component.id = id;
                }
                if let (Some(id), Some(component)) = (material_id, material.as_deref_mut()) {
                    component.id = id;
                }
                if has_transform_options {
                    let updated =
                        parse_transform_options(&options, transform.as_deref().unwrap().clone())?;
                    *transform.as_deref_mut().unwrap() = updated;
                }

                found = true;
                break;
            }

            if !found {
                return Err(format!("no entity named '{name}' exists"));
            }

            Ok(())
        })();

        match result {
            Ok(()) => emit(&mut commands, "Object updated", OutputLevel::Info),
            Err(error) => emit(
                &mut commands,
                format!("edit_object: {error}"),
                OutputLevel::Error,
            ),
        }
    }
}

pub struct AnimateObjectCommand;

impl AnimateObjectCommand {
    pub fn name() -> &'static str {
        "animate_object"
    }

    pub fn execute(
        arguments: In<Vec<String>>,
        objects: Query<(bevy_ecs::entity::Entity, &VFSName, &Transform)>,
        mut commands: Commands,
    ) {
        let result = (|| -> Result<_, String> {
            let options = parse_options(&arguments.0)?;

            let name = options
                .get("object")
                .ok_or_else(|| "missing --object".to_string())?;

            if !options.contains_key("position") && !options.contains_key("transform") {
                return Err("missing --transform (or --position)".to_string());
            }

            let duration = options
                .get("duration")
                .map(|value| parse_f32(value, "duration"))
                .transpose()?
                .unwrap_or(1.0);

            if duration <= 0.0 {
                return Err("--duration must be greater than zero".to_string());
            }

            let (entity, current) = objects
                .iter()
                .find(|(_, vfs_name, _)| vfs_name.name == *name)
                .map(|(entity, _, transform)| (entity, transform.clone()))
                .ok_or_else(|| format!("no entity named '{name}' exists"))?;

            let target = parse_transform_options(&options, current)?;

            Ok((entity, target, duration, name.to_string()))
        })();

        match result {
            Ok((entity, target, duration, name)) => {
                let current = objects
                    .get(entity)
                    .map(|(_, _, transform)| transform.clone());

                if let Ok(current) = current {
                    commands.entity(entity).insert(TransformAnimation {
                        target,
                        elapsed: 0.0,
                        duration,
                    });
                    let _ = current;
                    emit(
                        &mut commands,
                        format!("Started animation for '{name}' ({duration}s)"),
                        OutputLevel::Info,
                    );
                } else {
                    emit(
                        &mut commands,
                        format!("animate_object: no transform on '{name}'"),
                        OutputLevel::Error,
                    );
                }
            }
            Err(error) => emit(
                &mut commands,
                format!("animate_object: {error}"),
                OutputLevel::Error,
            ),
        }
    }
}

pub fn advance_animations(
    mut animations: Query<(
        bevy_ecs::entity::Entity,
        &mut Transform,
        &mut TransformAnimation,
    )>,
    dt: Res<FrameDelta>,
    mut commands: Commands,
) {
    let delta = dt.0.max(0.0);

    for (entity, mut transform, mut animation) in &mut animations {
        animation.elapsed = (animation.elapsed + delta).min(animation.duration);
        let t = (animation.elapsed / animation.duration).clamp(0.0, 1.0);

        transform.translation = transform.translation.lerp(animation.target.translation, t);
        transform.scale = transform.scale.lerp(animation.target.scale, t);
        transform.rotation = transform.rotation.slerp(animation.target.rotation, t);

        if t >= 1.0 {
            commands.entity(entity).remove::<TransformAnimation>();
        }
    }
}

pub struct AddLightCommand;

impl AddLightCommand {
    pub fn name() -> &'static str {
        "add_light"
    }

    pub fn execute(
        arguments: In<Vec<String>>,
        existing_names: Query<&VFSName>,
        mut commands: Commands,
    ) {
        let result = (|| -> Result<_, String> {
            let options = parse_options(&arguments.0)?;

            let name = options
                .get("name")
                .ok_or_else(|| "missing --name".to_string())?
                .to_string();
            ensure_name_is_available(&name, &existing_names)?;

            let color = parse_vec3(
                options
                    .get("color")
                    .ok_or_else(|| "missing --color (r,g,b)".to_string())?,
                "color",
            )?;
            let xyz = parse_vec3(
                options
                    .get("xyz")
                    .ok_or_else(|| "missing --xyz".to_string())?,
                "xyz",
            )?;
            let range = parse_f32(
                options
                    .get("range")
                    .ok_or_else(|| "missing --range".to_string())?,
                "range",
            )?;
            let intensity = parse_f32(
                options
                    .get("intensity")
                    .ok_or_else(|| "missing --intensity".to_string())?,
                "intensity",
            )?;

            if range <= 0.0 {
                return Err("--range must be greater than zero".to_string());
            }
            if intensity < 0.0 {
                return Err("--intensity cannot be negative".to_string());
            }

            Ok((name, color, xyz, range, intensity))
        })();

        match result {
            Ok((name, color, xyz, range, intensity)) => {
                commands.spawn((
                    VFSName { name: name.clone() },
                    Light {
                        color: color.to_array(),
                        range,
                        intensity,
                    },
                    Transform {
                        translation: xyz,
                        ..default_transform()
                    },
                ));

                emit(
                    &mut commands,
                    format!("Created light '{name}'"),
                    OutputLevel::Info,
                );
            }
            Err(error) => emit(
                &mut commands,
                format!("add_light: {error}"),
                OutputLevel::Error,
            ),
        }
    }
}

pub struct AddPointCommand;

impl AddPointCommand {
    pub fn name() -> &'static str {
        "add_point"
    }

    pub fn execute(
        arguments: In<Vec<String>>,
        builtins: Res<BuiltinAssets>,
        existing_names: Query<&VFSName>,
        mut commands: Commands,
    ) {
        let result = (|| -> Result<_, String> {
            let options = parse_options(&arguments.0)?;

            let name = options
                .get("name")
                .ok_or_else(|| "missing --name".to_string())?
                .to_string();
            ensure_name_is_available(&name, &existing_names)?;

            let xyz = parse_vec3(
                options
                    .get("xyz")
                    .ok_or_else(|| "missing --xyz".to_string())?,
                "xyz",
            )?;
            let size = options
                .get("size")
                .map(|value| parse_f32(value, "size"))
                .transpose()?
                .unwrap_or(0.1);

            if size <= 0.0 {
                return Err("--size must be greater than zero".to_string());
            }

            let material_name = options.get("material").copied().unwrap_or("white_matte");
            let material_id = resolve_material(material_name, &builtins)?;

            Ok((name, xyz, size, material_id))
        })();

        match result {
            Ok((name, xyz, size, material_id)) => {
                commands.spawn((
                    VFSName { name: name.clone() },
                    Mesh {
                        id: builtins.meshes.sphere,
                    },
                    Material { id: material_id },
                    Transform {
                        translation: xyz,
                        scale: glam::Vec3::splat(size),
                        ..default_transform()
                    },
                ));

                emit(
                    &mut commands,
                    format!("Created point '{name}'"),
                    OutputLevel::Info,
                );
            }
            Err(error) => emit(
                &mut commands,
                format!("add_point: {error}"),
                OutputLevel::Error,
            ),
        }
    }
}

pub struct AddLineCommand;

impl AddLineCommand {
    pub fn name() -> &'static str {
        "add_line"
    }

    pub fn execute(
        arguments: In<Vec<String>>,
        builtins: Res<BuiltinAssets>,
        existing_names: Query<&VFSName>,
        mut commands: Commands,
    ) {
        let result = (|| -> Result<_, String> {
            let options = parse_options(&arguments.0)?;

            let name = options
                .get("name")
                .ok_or_else(|| "missing --name".to_string())?
                .to_string();
            ensure_name_is_available(&name, &existing_names)?;

            let start = parse_vec3(
                options
                    .get("start")
                    .ok_or_else(|| "missing --start".to_string())?,
                "start",
            )?;
            let end = parse_vec3(
                options
                    .get("end")
                    .ok_or_else(|| "missing --end".to_string())?,
                "end",
            )?;

            let direction = end - start;
            let length = direction.length();
            if length <= f32::EPSILON {
                return Err("--start and --end must be different points".to_string());
            }

            let width = options
                .get("width")
                .map(|value| parse_f32(value, "width"))
                .transpose()?
                .unwrap_or(0.025);
            if width <= 0.0 {
                return Err("--width must be greater than zero".to_string());
            }

            let material_name = options.get("material").copied().unwrap_or("white_matte");
            let material_id = resolve_material(material_name, &builtins)?;

            let rotation = glam::Quat::from_rotation_arc(glam::Vec3::Y, direction / length);

            Ok((
                name,
                (start + end) * 0.5,
                glam::Vec3::new(width, length, width),
                rotation,
                material_id,
            ))
        })();

        match result {
            Ok((name, midpoint, scale, rotation, material_id)) => {
                commands.spawn((
                    VFSName { name: name.clone() },
                    Mesh {
                        id: builtins.meshes.cylinder,
                    },
                    Material { id: material_id },
                    Transform {
                        translation: midpoint,
                        scale,
                        rotation,
                    },
                ));

                emit(
                    &mut commands,
                    format!("Created line '{name}'"),
                    OutputLevel::Info,
                );
            }
            Err(error) => emit(
                &mut commands,
                format!("add_line: {error}"),
                OutputLevel::Error,
            ),
        }
    }
}
