use eframe::egui;
use eframe::egui::Theme;
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

/// Margin to be applied to the main frame of the application.
pub const FRAME_MARGIN: f32 = 50.0;

/// Corner radius for the input fields.
pub const INPUT_CORNER_RADIUS: u8 = 6;

/// Background color for the UI.
pub const FRAME_BG_COLOR: egui::Color32 = egui::Color32::from_rgb(250, 250, 250);

/// Background color for the UI.
pub const FRAME_BG_DARK_COLOR: egui::Color32 = egui::Color32::from_rgb(250, 250, 250);

/// Text color for the UI.
pub const FRAME_TEXT_COLOR: egui::Color32 = egui::Color32::from_rgb(31, 11, 53);

/// Background color for the buttons.
pub const BUTTON_BG_COLOR: egui::Color32 = egui::Color32::from_rgb(42, 255, 186);

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
pub fn render_title(ui: &mut egui::Ui, ctx: &egui::Context, text: &str) {
    render_heading(ui, ctx, text, TITLE_SIZE);
}

/// Renders a subtitle-styled label with a specific text.
pub fn render_subtitle(ui: &mut egui::Ui, ctx: &egui::Context, text: &str) {
    render_heading(ui, ctx, text, SUBTITLE_SIZE);
}

/// Renders a styled input field with a given label and control text.
pub fn render_input(
    ui: &mut egui::Ui,
    label: &str,
    text: &mut String,
    is_password: bool,
    text_hint: Option<&str>,
) {
    ui.add_space(WIDGET_SPACING_BASE);
    ui.label(RichText::new(label));

    let mut edit_text = egui::TextEdit::singleline(text)
        .password(is_password)
        .desired_width(INPUT_WIDTH);
    if let Some(hint) = text_hint {
        edit_text = edit_text.hint_text(hint);
    }
    ui.add(edit_text);
    ui.add_space(WIDGET_SPACING_BASE);
}

pub fn render_base_input(text: &mut String, is_password: bool, _use_frame: bool) -> egui::TextEdit {
    egui::TextEdit::singleline(text)
        .password(is_password)
        .desired_width(INPUT_WIDTH)
}

pub fn render_button(ui: &mut egui::Ui, ctx: &egui::Context, label: &str, callback: impl FnOnce()) {
    let theme = ctx.theme();

    ui.spacing_mut().button_padding =
        egui::vec2(4.0 * WIDGET_SPACING_BASE, 2.0 * WIDGET_SPACING_BASE);

    let text_label = match theme {
        Theme::Dark => RichText::new(label),
        Theme::Light => RichText::new(label).color(FRAME_TEXT_COLOR),
    };
    let button = match theme {
        Theme::Dark => egui::Button::new(text_label),
        Theme::Light => egui::Button::new(text_label).fill(BUTTON_BG_COLOR),
    };

    if ui.add(button).clicked() {
        callback();
    }
}

pub fn render_unaligned_button(ui: &mut egui::Ui, label: &str, callback: impl FnOnce()) {
    if ui.button(label).clicked() {
        callback();
    }
}

/// Renders a heading-styled label with a specific text and size.
fn render_heading(ui: &mut egui::Ui, ctx: &egui::Context, text: &str, size: f32) {
    match ctx.theme() {
        Theme::Dark => {
            egui::Frame::default()
                .inner_margin(egui::vec2(size / 2.0, size / 2.0))
                .show(ui, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(
                            RichText::new(text)
                                .text_style(egui::TextStyle::Heading)
                                .size(size)
                                .strong(),
                        );
                    });
                });
        }
        Theme::Light => {
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
    }
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
