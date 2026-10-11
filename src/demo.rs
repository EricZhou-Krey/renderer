use crate::{
    asset::loader::RequestLoadScene,
    component::{camera::ActiveCamera, Transform},
    scene_sheet::demo1_scene_source,
};
use bevy_ecs::{schedule::Schedule, world::World};

pub trait DemoWorldExtension {
    fn setup_demo(&mut self);
}

impl DemoWorldExtension for World {
    fn setup_demo(&mut self) {
        self.trigger(RequestLoadScene {
            scene: demo1_scene_source(),
        });
    }
}

pub trait DemoScheduleExtension {
    fn setup_demo(&mut self);
}

impl DemoScheduleExtension for Schedule {
    fn setup_demo(&mut self) {}
}

pub fn move_camera_from_keyboard(world: &mut World, ctx: &egui::Context, dt: f32) {
    let wants_keyboard = ctx.egui_wants_keyboard_input();

    let (forward, backward, left, right, up, down, look_inactive, mouse_delta, fast) =
        ctx.input(|input| {
            let look_active =
                input.pointer.button_down(egui::PointerButton::Secondary) && !wants_keyboard;

            (
                input.key_down(egui::Key::W),
                input.key_down(egui::Key::S),
                input.key_down(egui::Key::A),
                input.key_down(egui::Key::D),
                input.key_down(egui::Key::E),
                input.key_down(egui::Key::Q),
                look_active,
                input
                    .pointer
                    .motion()
                    .unwrap_or_else(|| input.pointer.delta()),
                input.modifiers.shift,
            )
        });

    ctx.send_viewport_cmd(egui::ViewportCommand::CursorGrab(if !look_inactive {
        egui::viewport::CursorGrab::Locked
    } else {
        egui::viewport::CursorGrab::None
    }));
    ctx.send_viewport_cmd(egui::ViewportCommand::CursorVisible(look_inactive));

    if wants_keyboard || look_inactive {
        return;
    }

    let Some(camera_entity) = world
        .get_resource::<ActiveCamera>()
        .map(|active| active.camera)
    else {
        return;
    };

    let Some(mut transform) = world.get_mut::<Transform>(camera_entity) else {
        return;
    };

    let speed = if fast { 12.0 } else { 4.0 };
    let mouse_sensitivity = 0.003;

    let yaw = -mouse_delta.x * mouse_sensitivity;
    let pitch = mouse_delta.y * mouse_sensitivity;

    transform.rotation *= glam::Quat::from_rotation_y(yaw) * glam::Quat::from_rotation_x(pitch);

    let local_direction = glam::Vec3::new(
        right as i32 as f32 - left as i32 as f32,
        up as i32 as f32 - down as i32 as f32,
        backward as i32 as f32 - forward as i32 as f32,
    );

    if local_direction != glam::Vec3::ZERO {
        let direction = (transform.rotation * local_direction).normalize();
        transform.translation += direction * speed * dt;
    }
}
