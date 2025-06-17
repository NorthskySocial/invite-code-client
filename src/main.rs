#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod home_page;
mod login_page;
mod qr_validate_page;
mod qr_verify_page;

use crate::home_page::HomePage;
use crate::login_page::LoginPage;
use crate::qr_validate_page::QrValidatePage;
use crate::qr_verify_page::QrVerifyPage;
use eframe::egui;
use reqwest::Client;
#[cfg(not(target_arch = "wasm32"))]
use reqwest::cookie::Jar;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::Arc;
use std::sync::mpsc::Receiver;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Duration;
#[cfg(not(target_arch = "wasm32"))]
use tokio::runtime::Runtime;

mod styles;
mod util;

enum Page {
    Home(HomePage),
    Login(LoginPage),
    QrVerify(QrVerifyPage),
    QrValidate(QrValidatePage),
}

const LOGIN: &str = "/api/auth/login";
const GET_INVITE_CODES: &str = "/api/invite-codes";
const GENERATE_OTP: &str = "/api/auth/otp/generate";
const VALIDATE_OTP: &str = "/api/auth/otp/validate";
const VERIFY_OTP: &str = "/api/auth/otp/verify";
const CREATE_INVITE_CODES: &str = "/api/create-invite-codes";
const DISABLE_INVITE_CODES: &str = "/api/disable-invite-codes";

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

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default(),
        ..Default::default()
    };

    eframe::run_native(
        "Invite Code Manager",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            styles::setup_fonts(&cc.egui_ctx);
            Ok(Box::<InviteCodeManager>::default())
        }),
    )
}

// When compiling to web using trunk:
#[cfg(target_arch = "wasm32")]
fn main() {
    use eframe::wasm_bindgen::JsCast as _;
    // Redirect `log` message to `console.log` and friends:
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

struct InviteCodeManager {
    page: Page,
    page_rx: Receiver<Page>,
}

impl Default for InviteCodeManager {
    #[cfg(not(target_arch = "wasm32"))]
    fn default() -> Self {
        let (page_tx, page_rx) = std::sync::mpsc::channel();
        let cookie_store = Arc::new(Jar::default());
        let client = Client::builder()
            .cookie_store(true)
            .cookie_provider(cookie_store.clone())
            .build()
            .unwrap();
        Self {
            page: Page::Login(LoginPage::new(
                page_tx.clone(),
                client,
                "https://invites.northsky.social".to_string(),
            )),
            page_rx,
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn default() -> Self {
        let (page_tx, page_rx) = std::sync::mpsc::channel();
        let client = Client::builder().build().unwrap();
        Self {
            page: Page::Login(LoginPage::new(
                page_tx.clone(),
                client,
                "https://invites.northsky.social".to_string(),
            )),
            page_rx,
        }
    }
}

impl eframe::App for InviteCodeManager {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Setup some application window properties through the `egui` frame.
        let styled_frame = styles::get_styled_frame();

        egui::CentralPanel::default()
            .frame(styled_frame)
            .show(ctx, |ui| {
                // Basic window styling.
                styles::set_text_color(ui);
                styles::render_title(ui, styles::FRAME_TITLE);

                let res = self.page_rx.try_recv();
                if res.is_ok() {
                    self.page = res.unwrap();
                }

                match &mut self.page {
                    Page::Home(home_page) => {
                        home_page.show(ui);
                    }
                    Page::Login(login_page) => {
                        login_page.show(ui);
                    }
                    Page::QrVerify(verify_page) => {
                        verify_page.show(ui);
                    }
                    Page::QrValidate(validate_page) => {
                        validate_page.show(ui);
                    }
                }
            });
    }
}
