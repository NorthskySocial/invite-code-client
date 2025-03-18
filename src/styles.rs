use egui::RichText;

/// Margin to be applied to the main frame of the application.
pub const FRAME_MARGIN: f32 = 50.0;

/// Background color for the UI.
pub const FRAME_BG_COLOR: egui::Color32 = egui::Color32::from_rgb(222, 229, 229);

/// Text color for the UI.
pub const FRAME_TEXT_COLOR: egui::Color32 = egui::Color32::from_rgb(25, 9, 42);

/// Title of the application frame.
pub const FRAME_TITLE: &str = "Northsky Invite Codes Tool";

/// Size of the title text.
pub const TITLE_SIZE: f32 = 36.0;

/// Size of the subtitle text.
pub const SUBTITLE_SIZE: f32 = 24.0;

/// Returns a frame with styles applied to be used as the main application frame.
pub fn get_styled_frame() -> egui::Frame {
    egui::Frame::new()
        .inner_margin(egui::vec2(FRAME_MARGIN, FRAME_MARGIN))
        .fill(FRAME_BG_COLOR)
}

/// Sets the UI text color.
pub fn set_text_color(ui: &mut egui::Ui) {
    ui.visuals_mut().override_text_color = Some(FRAME_TEXT_COLOR);
}

/// Renders a title-styled label with a specific text.
pub fn render_title(ui: &mut egui::Ui, text: &str) {
    render_heading(ui, text, TITLE_SIZE);
}

/// Renders a subtitle-styled label with a specific text.
pub fn render_subtitle(ui: &mut egui::Ui, text: &str) {
    render_heading(ui, text, SUBTITLE_SIZE);
}

/// Renders a heading-styled label with a specific text and size.
fn render_heading(ui: &mut egui::Ui, text: &str, size: f32) {
    egui::Frame::default()
        .inner_margin(egui::vec2(size / 2.0, size / 2.0))
        .show(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(
                    RichText::new(text)
                        .text_style(egui::TextStyle::Heading)
                        .size(size)
                        .color(FRAME_TEXT_COLOR)
                        .strong(),
                );
            });
        });
}
