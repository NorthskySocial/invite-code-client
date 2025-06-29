use eframe::egui;
use egui::{CornerRadius, RichText};

/// Title of the application frame.
pub const FRAME_TITLE: &str = "Invite Code Manager";

/// Margin to be applied to the main frame of the application.
const FRAME_MARGIN: f32 = 50.0;

/// Corner radius for the input fields.
const INPUT_CORNER_RADIUS: u8 = 6;

/// Background color for the UI.
const FRAME_BG_COLOR: egui::Color32 = egui::Color32::from_rgb(250, 250, 250);

/// Text color for the UI.
const FRAME_TEXT_COLOR: egui::Color32 = egui::Color32::from_rgb(31, 11, 53);

/// Text color for the error messages.
const ERROR_TEXT_COLOR: egui::Color32 = egui::Color32::from_rgb(255, 0, 0);

/// Size of the title text.
const TITLE_SIZE: f32 = 36.0;

/// Size of the subtitle text.
const SUBTITLE_SIZE: f32 = 24.0;

/// Background color for the buttons.
const BUTTON_BG_COLOR: egui::Color32 = egui::Color32::from_rgb(42, 255, 186);

/// Input field width.
const INPUT_WIDTH: f32 = 200.0;

/// Base measure to be used for different spacing calculations in the UI.
const WIDGET_SPACING_BASE: f32 = 5.0;

/// Font name for the main UI font.
const MAIN_FONT_NAME: &str = "Geist";

/// Returns a frame with styles applied to be used as the main application frame.
pub fn get_styled_frame() -> egui::Frame {
    egui::Frame::new()
        .inner_margin(egui::vec2(FRAME_MARGIN, FRAME_MARGIN))
        .fill(FRAME_BG_COLOR)
}

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

/// Renders a styled input field with a given label and control text.
pub fn render_input(ui: &mut egui::Ui, label: &str, text: &mut String, is_password: bool) {
    ui.vertical_centered(|ui| {
        ui.add_space(WIDGET_SPACING_BASE);
        ui.label(RichText::new(label).color(FRAME_TEXT_COLOR));

        egui::Frame::new()
            .stroke(egui::Stroke::new(1.0, FRAME_TEXT_COLOR))
            .corner_radius(egui::CornerRadius::same(INPUT_CORNER_RADIUS))
            .inner_margin(egui::vec2(WIDGET_SPACING_BASE, WIDGET_SPACING_BASE))
            .show(ui, |ui| {
                ui.set_max_width(INPUT_WIDTH * 1.10);
                ui.add(render_base_input(text, is_password, false));
            });
        ui.add_space(WIDGET_SPACING_BASE);
    });
}

pub fn render_base_input(text: &mut String, is_password: bool, use_frame: bool) -> egui::TextEdit {
    egui::TextEdit::singleline(text)
        .text_color(FRAME_TEXT_COLOR)
        .background_color(FRAME_BG_COLOR)
        .password(is_password)
        .frame(use_frame)
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
    ui.spacing_mut().button_padding =
        egui::vec2(4.0 * WIDGET_SPACING_BASE, 2.0 * WIDGET_SPACING_BASE);

    let text_label = egui::RichText::new(label).color(FRAME_TEXT_COLOR);
    let button = egui::Button::new(text_label)
        .fill(BUTTON_BG_COLOR)
        .corner_radius(CornerRadius::same(0));

    if ui.add(button).clicked() {
        callback();
    }
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
