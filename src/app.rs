use crate::{
    asset::AssetWorldExtension,
    commands::advance_animations,
    component::{
        camera::{ActiveCamera, Camera, CameraProjection},
        Transform,
    },
    demo::setup_demo_scene,
    renderer::{resource::init_callback_resources, Renderer},
    resource::{FrameDelta, ResoureWorldExtension},
    terminal::RendererTerminalWorldExtension,
};
use bevy_ecs::{entity::Entity, schedule::Schedule, world::World};
use eframe::CreationContext;
use glam::{Quat, Vec3};
use rook_terminal::{
    command::HelpCommand, event::TerminalCommandRequested, TerminalInputState, TerminalSession,
    TerminalViewState, TerminalWorldExtension,
};

pub struct RaytraceApp {
    pub world: World,
    pub schedule: Schedule,
}

impl RaytraceApp {
    pub fn new(cc: &CreationContext) -> Self {
        let mut world: World = World::new();
        world.setup_terminal();
        world.setup_render_terminal();
        world.setup_assets();
        world.setup_resource();
        setup_demo_scene(&mut world);

        let camera: Entity = world
            .spawn((
                Camera,
                Transform {
                    translation: Vec3::ZERO,
                    rotation: Quat::IDENTITY,
                    scale: Vec3::ONE,
                },
            ))
            .id();

        world.insert_resource(ActiveCamera { camera });
        world.insert_resource(CameraProjection {
            vertical_fov: 60.0_f32.to_radians(),
            aspect_ratio: 16.0 / 9.0,
            z_near: 0.1,
            z_far: 1000.0,
        });

        world.trigger(TerminalCommandRequested {
            raw_command: HelpCommand::name().to_string(),
        });
        world.flush();

        init_callback_resources(cc, &mut world);

        let mut schedule: Schedule = Schedule::default();
        schedule.add_systems(advance_animations);

        Self { world, schedule }
    }
}

impl eframe::App for RaytraceApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::left("Terminal").show(ui, |ui: &mut egui::Ui| {
            let pending_event: Option<TerminalCommandRequested> = self
                .world
                .resource_scope::<TerminalSession, _>(|world, mut terminal_session| {
                    world.resource_scope::<TerminalInputState, _>(|world, mut input_state| {
                        let view_state = world.resource::<TerminalViewState>();

                        terminal_session.ui(&mut input_state, view_state, ui)
                    })
                });

            if let Some(event) = pending_event {
                self.world.trigger(event);
                self.world.flush();
            }
        });

        egui::CentralPanel::default().show(ui, |ui: &mut egui::Ui| {
            Renderer::ui(ui, &mut self.world);
        });
    }

    fn logic(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let dt: f32 = ctx.input(|input| input.stable_dt.min(0.1));

        self.world.resource_mut::<FrameDelta>().0 = dt;
        move_camera_from_keyboard(&mut self.world, ctx, dt);
        self.schedule.run(&mut self.world);

        ctx.request_repaint();
    }
}

pub fn move_camera_from_keyboard(world: &mut World, ctx: &egui::Context, dt: f32) {
    let wants_keyboard = ctx.egui_wants_keyboard_input();

    let (forward, backward, left, right, up, down, look_active, mouse_delta, fast) =
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

    ctx.send_viewport_cmd(egui::ViewportCommand::CursorGrab(if look_active {
        egui::viewport::CursorGrab::Locked
    } else {
        egui::viewport::CursorGrab::None
    }));
    ctx.send_viewport_cmd(egui::ViewportCommand::CursorVisible(!look_active));

    if wants_keyboard || !look_active {
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
