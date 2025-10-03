#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use crate::app::InviteCodeManager;
#[cfg(not(target_arch = "wasm32"))]
use eframe::egui;
use std::sync::Arc;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Duration;
#[cfg(not(target_arch = "wasm32"))]
use tokio::runtime::Runtime;

mod app;
mod styles;
mod util;

enum Page {
    Home,
    Login,
    QrVerify,
    QrValidate,
}

const LOGIN: &str = "/auth/login";
const GET_INVITE_CODES: &str = "/invite-codes";
const GENERATE_OTP: &str = "/auth/otp/generate";
const VALIDATE_OTP: &str = "/auth/otp/validate";
const VERIFY_OTP: &str = "/auth/otp/verify";
const CREATE_INVITE_CODES: &str = "/create-invite-codes";
const DISABLE_INVITE_CODES: &str = "/disable-invite-codes";

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let rt = Runtime::new().expect("Unable to create Runtime");

    // Enter the runtime so that `tokio::spawn` is available immediately.
    let _enter = rt.enter();

    // Execute the runtime in its own thread.
    // The future doesn't have to do anything. In this example, it just sleeps forever.
    std::thread::spawn(move || {
        rt.block_on(async {
            loop {
                tokio::time::sleep(Duration::from_secs(3600)).await;
            }
        })
    });

    let icon_data =
        eframe::icon_data::from_png_bytes(include_bytes!("../assets/apple-touch-icon.png"))
            .expect("The icon data must be valid");

    let options = eframe::NativeOptions {
        viewport: {
            egui::ViewportBuilder {
                icon: Some(Arc::new(icon_data)),
                ..Default::default()
            }
        },
        ..Default::default()
    };

    eframe::run_native(
        "Invite Code Manager",
        options,
        Box::new(|cc| Ok(Box::new(InviteCodeManager::new(cc)))),
    )
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Failed to find the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement");

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| {
                    egui_extras::install_image_loaders(&cc.egui_ctx);
                    styles::setup_fonts(&cc.egui_ctx);
                    Ok(Box::<InviteCodeManager>::default())
                }),
            )
            .await;

        // Remove the loading text and spinner:
        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });
}
