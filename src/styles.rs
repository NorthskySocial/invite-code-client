use eframe::egui;
use eframe::egui::Theme;
use egui::RichText;

/// Title of the application frame.
pub const FRAME_TITLE: &str = "Invite Code Manager";
/// Text color for the error messages.
const ERROR_TEXT_COLOR: egui::Color32 = egui::Color32::from_rgb(255, 0, 0);

/// Size of the title text.
fn get_title_size(ctx: &egui::Context) -> f32 {
    let width = get_screen_width(ctx);
    if is_mobile(ctx) {
        (width / 12.0).clamp(1.0, 40.0)
    } else {
        36.0
    }
}

/// Size of the subtitle text.
fn get_subtitle_size(ctx: &egui::Context) -> f32 {
    let width = get_screen_width(ctx);
    if is_mobile(ctx) {
        (width / 18.0).clamp(1.0, 28.0)
    } else {
        24.0
    }
}

/// Input field width.
const INPUT_WIDTH: f32 = 200.0;

/// Base measure to be used for different spacing calculations in the UI.
const WIDGET_SPACING_BASE: f32 = 8.0;
/// Font name for the main UI font.
const MAIN_FONT_NAME: &str = "Geist";

pub fn is_mobile(ctx: &egui::Context) -> bool {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window() {
            if let Ok(user_agent) = window.navigator().user_agent() {
                let ua = user_agent.to_lowercase();
                if ua.contains("iphone")
                    || ua.contains("ipad")
                    || ua.contains("android")
                    || ua.contains("mobi")
                {
                    return true;
                }
            }
        }
    }

    get_screen_width(ctx) < 600.0
}

pub fn get_screen_width(ctx: &egui::Context) -> f32 {
    ctx.input(|i| {
        i.viewport()
            .outer_rect
            .map(|rect| rect.width())
            .unwrap_or_else(|| i.content_rect().width())
    })
}

pub fn get_dynamic_zoom_factor(ctx: &egui::Context) -> f32 {
    if !is_mobile(ctx) {
        return 1.0;
    }

    let width = get_screen_width(ctx);

    // Baseline width for mobile scaling (e.g., iPhone SE-ish width)
    let baseline_width = 375.0;

    // We want a zoom factor that scales with width, but doesn't go too crazy.
    // If width is 375, zoom is 1.0.
    // If width is 320, zoom is (320/375) * 1.0 = 0.85.
    (width / baseline_width).clamp(0.8, 1.2)
}

/// Text color for the UI.
pub const FRAME_TEXT_COLOR: egui::Color32 = egui::Color32::from_rgb(31, 11, 53);

/// Background color for the buttons.
pub const BUTTON_BG_COLOR: egui::Color32 = egui::Color32::from_rgb(42, 255, 186);

pub fn apply_global_style(ctx: &egui::Context) {
    let mut visuals = egui::Visuals::dark();

    // Customize visuals
    visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(8);
    visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(8);
    visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(8);
    visuals.widgets.active.corner_radius = egui::CornerRadius::same(8);
    visuals.widgets.open.corner_radius = egui::CornerRadius::same(8);

    visuals.window_corner_radius = egui::CornerRadius::same(12);
    visuals.window_shadow = egui::Shadow {
        blur: 20,
        color: egui::Color32::from_black_alpha(80),
        ..Default::default()
    };

    ctx.set_visuals(visuals);
}

pub fn render_card<R>(
    ui: &mut egui::Ui,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::InnerResponse<R> {
    let margin = if is_mobile(ui.ctx()) {
        egui::Margin::symmetric(4, 4)
    } else {
        egui::Margin::symmetric(24, 8)
    };

    egui::Frame::group(ui.style())
        .fill(ui.visuals().faint_bg_color)
        .corner_radius(egui::CornerRadius::same(8))
        .inner_margin(if is_mobile(ui.ctx()) {
            egui::Margin::same(8)
        } else {
            egui::Margin::same(16)
        })
        .outer_margin(margin)
        .show(ui, add_contents)
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
pub fn set_text_color(_ui: &mut egui::Ui) {
    // ui.visuals_mut().override_text_color = Some(FRAME_TEXT_COLOR);
}

/// Renders a title-styled label with a specific text.
pub fn render_title(ui: &mut egui::Ui, ctx: &egui::Context, text: &str) {
    render_heading(ui, ctx, text, get_title_size(ctx));
}

/// Renders a subtitle-styled label with a specific text.
pub fn render_subtitle(ui: &mut egui::Ui, ctx: &egui::Context, text: &str) {
    render_heading(ui, ctx, text, get_subtitle_size(ctx));
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
    ui.label(RichText::new(label).strong());

    let ctx = ui.ctx().clone();
    let height = if is_mobile(&ctx) { 32.0 } else { 30.0 };

    let mut edit_text = egui::TextEdit::singleline(text)
        .password(is_password)
        .desired_width(if is_mobile(&ctx) {
            ui.available_width() - 40.0
        } else {
            f32::INFINITY
        }) // More conservative width on mobile
        .margin(egui::Margin::symmetric(8, 12));

    if let Some(hint) = text_hint {
        edit_text = edit_text.hint_text(hint);
    }
    ui.add_sized(
        [
            if is_mobile(&ctx) {
                ui.available_width() - 40.0
            } else {
                INPUT_WIDTH
            },
            height,
        ],
        edit_text,
    );
    ui.add_space(WIDGET_SPACING_BASE);
}

pub fn render_base_input(
    text: &mut String,
    is_password: bool,
    _use_frame: bool,
) -> egui::TextEdit<'_> {
    egui::TextEdit::singleline(text)
        .password(is_password)
        .desired_width(INPUT_WIDTH)
        .margin(egui::Margin::symmetric(8, 4))
}

pub fn render_button(ui: &mut egui::Ui, ctx: &egui::Context, label: &str, callback: impl FnOnce()) {
    let theme = ctx.theme();

    let padding = if is_mobile(ctx) {
        egui::vec2(16.0, 12.0)
    } else {
        egui::vec2(2.0 * WIDGET_SPACING_BASE, WIDGET_SPACING_BASE)
    };
    ui.spacing_mut().button_padding = padding;

    let text_label = match theme {
        Theme::Dark => RichText::new(label).strong(),
        Theme::Light => RichText::new(label).color(FRAME_TEXT_COLOR).strong(),
    };

    let text_label = if is_mobile(ctx) {
        let width = get_screen_width(ctx);
        let size = (width / 20.0).clamp(16.0, 20.0);
        text_label.size(size)
    } else {
        text_label
    };

    let button = match theme {
        Theme::Dark => egui::Button::new(text_label),
        Theme::Light => egui::Button::new(text_label).fill(BUTTON_BG_COLOR),
    };

    let button = if is_mobile(ctx) {
        button.min_size(egui::vec2(ui.available_width() - 20.0, 48.0)) // Increased from 44.0
    } else {
        button
    };

    let response = ui.add(button);
    if response.clicked() {
        callback();
    }
}

/// Renders a heading-styled label with a specific text and size.
fn render_heading(ui: &mut egui::Ui, ctx: &egui::Context, text: &str, size: f32) {
    let margin = if is_mobile(ctx) {
        size / 6.0
    } else {
        size / 2.0
    };
    match ctx.theme() {
        Theme::Dark => {
            egui::Frame::default()
                .inner_margin(egui::vec2(margin, margin))
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
                .inner_margin(egui::vec2(margin, margin))
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
