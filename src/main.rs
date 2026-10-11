use eframe::{CreationContext, NativeOptions};
use renderer::app::RaytraceApp;

fn main() -> eframe::Result {
    env_logger::init();

    let native_options: NativeOptions = NativeOptions {
        viewport: egui::ViewportBuilder::default().with_maximized(true),
        renderer: eframe::Renderer::Wgpu,
        depth_buffer: 32,
        ..Default::default()
    };

    eframe::run_native(
        "sound_raytrace",
        native_options,
        Box::new(|cc: &CreationContext| Ok(Box::new(RaytraceApp::new(cc)))),
    )
}
