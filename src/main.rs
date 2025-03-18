#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod home_page;
mod login_page;
mod qr_validate_page;
mod qr_verify_page;

use crate::home_page::HomePage;
use crate::login_page::LoginPage;
use crate::qr_validate_page::QrValidatePage;
use crate::qr_verify_page::QrVerifyPage;
use reqwest::Client;
use reqwest::cookie::Jar;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;
use tokio::runtime::Runtime;

mod styles;

enum Page {
    Home(HomePage),
    Login(LoginPage),
    QrVerify(QrVerifyPage),
    QrValidate(QrValidatePage),
}

const LOGIN: &str = "/auth/login";
const GET_INVITE_CODES: &str = "/invite-codes";
const GENERATE_OTP: &str = "/auth/otp/generate";
const VALIDATE_OTP: &str = "/auth/otp/validate";
const VERIFY_OTP: &str = "/auth/otp/verify";
const CREATE_INVITE_CODES: &str = "/create-invite-codes";
const DISABLE_INVITE_CODES: &str = "/disable-invite-codes";

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
            Ok(Box::<InviteCodeManager>::default())
        }),
    )
}

struct InviteCodeManager {
    page: Page,
    invite_backend: String,

    // Sender/Receiver for async page
    page_tx: Sender<Page>,
    page_rx: Receiver<Page>,
}

impl Default for InviteCodeManager {
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
                "https://pds.example.com".to_string(),
            )),
            invite_backend: "https://pds.example.com".to_string(),
            page_tx,
            page_rx,
        }
    }
}

impl eframe::App for InviteCodeManager {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Setup some application window properties through the `egui` frame.
        let styled_frame = styles::get_styled_frame();

        egui::CentralPanel::default().frame(styled_frame).show(ctx, |ui| {
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
