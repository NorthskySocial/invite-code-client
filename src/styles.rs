use eframe::egui;
use egui::RichText;

/// Title of the application frame.
pub const FRAME_TITLE: &str = "Invite Code Manager";
/// Text color for the error messages.
const ERROR_TEXT_COLOR: egui::Color32 = egui::Color32::from_rgb(255, 0, 0);

/// Size of the title text.
const TITLE_SIZE: f32 = 36.0;

/// Size of the subtitle text.
const SUBTITLE_SIZE: f32 = 24.0;
/// Input field width.
const INPUT_WIDTH: f32 = 200.0;

/// Base measure to be used for different spacing calculations in the UI.
const WIDGET_SPACING_BASE: f32 = 5.0;

/// Font name for the main UI font.
const MAIN_FONT_NAME: &str = "Geist";

/// Sets up the fonts for the application using the `egui` context.
pub fn setup_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    fonts.font_data.insert(
        MAIN_FONT_NAME.to_owned(),
        egui::FontData::from_static(include_bytes!("../assets/Geist-VariableFont_wght.ttf")).into(),
    );

    fonts
        .families
        .entry(egui::FontFamily::Proportional)
        .or_default()
        .insert(0, MAIN_FONT_NAME.to_owned());

    ctx.set_fonts(fonts);
}

/// Sets the UI text color.
pub fn set_text_color(_ui: &mut egui::Ui) {
    // ui.visuals_mut().override_text_color = Some(FRAME_TEXT_COLOR);
}

/// Renders a title-styled label with a specific text.
pub fn render_title(ui: &mut egui::Ui, text: &str) {
    render_heading(ui, text, TITLE_SIZE);
}

/// Renders a subtitle-styled label with a specific text.
pub fn render_subtitle(ui: &mut egui::Ui, text: &str) {
    render_heading(ui, text, SUBTITLE_SIZE);
}

/// Renders a styled input field with a given label and control text.
pub fn render_input(ui: &mut egui::Ui, label: &str, text: &mut String, is_password: bool) {
    ui.vertical_centered(|ui| {
        ui.add_space(WIDGET_SPACING_BASE);
        ui.label(RichText::new(label));

        let edit_text = egui::TextEdit::singleline(text)
            .password(is_password)
            .desired_width(INPUT_WIDTH);
        ui.add(edit_text);
        ui.add_space(WIDGET_SPACING_BASE);
    });
}

pub fn render_base_input(text: &mut String, is_password: bool, _use_frame: bool) -> egui::TextEdit {
    egui::TextEdit::singleline(text)
        .password(is_password)
        .desired_width(INPUT_WIDTH)
}

/// Renders a styled button that runs a callback function when clicked.
pub fn render_button(ui: &mut egui::Ui, label: &str, callback: impl FnOnce()) {
    ui.add_space(WIDGET_SPACING_BASE);

    ui.vertical_centered(|ui| {
        render_unaligned_button(ui, label, callback);
    });

    ui.add_space(WIDGET_SPACING_BASE);
}

pub fn render_unaligned_button(ui: &mut egui::Ui, label: &str, callback: impl FnOnce()) {
    if ui.button(label).clicked() {
        callback();
    }
}

/// Renders a heading-styled label with a specific text and size.
fn render_heading(ui: &mut egui::Ui, text: &str, size: f32) {
    ui.vertical_centered(|ui| {
        ui.label(
            RichText::new(text)
                .text_style(egui::TextStyle::Heading)
                .size(size)
                .strong(),
        );
    });
}

pub fn render_error(ui: &mut egui::Ui, error_message: &str) {
    ui.add_space(WIDGET_SPACING_BASE);
    ui.vertical_centered(|ui| {
        ui.label(
            RichText::new(error_message)
                .color(ERROR_TEXT_COLOR)
                .strong(),
        );
    });
    ui.add_space(WIDGET_SPACING_BASE);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(FRAME_TITLE, "Invite Code Manager");
        assert_eq!(TITLE_SIZE, 36.0);
        assert_eq!(SUBTITLE_SIZE, 24.0);
        assert_eq!(INPUT_WIDTH, 200.0);
        assert_eq!(WIDGET_SPACING_BASE, 5.0);
        assert_eq!(MAIN_FONT_NAME, "Geist");
    }

    #[test]
    fn test_error_text_color() {
        assert_eq!(ERROR_TEXT_COLOR, egui::Color32::from_rgb(255, 0, 0));
    }
}
