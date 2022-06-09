use eframe::{NativeOptions, emath::Vec2, IconData};

mod editor;

const ICON_RESOURCE: &[u8] = include_bytes!("../logo.png");



fn main() {
    let app = editor::Editor::new();
    let mut native_options = NativeOptions::default();
    native_options.initial_window_size = Some(Vec2::new(1280.0, 720.0));

    if let Ok(icon) = image::load_from_memory_with_format(ICON_RESOURCE, image::ImageFormat::Png) {
        native_options.icon_data = Some(
            IconData {
                rgba: icon.as_bytes().to_vec(),
                width: icon.width(),
                height: icon.height(),
            }
        );
    }
    eframe::run_native("Rusty hex editor", native_options, Box::new(|cc| {
        Box::new(app)
    }));
}
