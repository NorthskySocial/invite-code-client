#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use egui_extras::{Column, TableBuilder};
use reqwest::cookie::Jar;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;
use tokio::runtime::Runtime;
use totp_rs::{Algorithm, Secret, TOTP};

mod styles;

enum Page {
    Home,
    Login,
    QrVerify,
    QrValidate,
}

#[derive(PartialEq, Eq, Default, Clone)]
pub struct QrCode {
    image: Vec<u8>,
    url: String,
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
    username: String,
    password: String,

    // cookie_store: Jar,
    client: Client,

    // Sender/Receiver for async notifications.
    tx: Sender<u32>,
    rx: Receiver<u32>,

    // Sender/Receiver for async invite codes.
    codes_tx: Sender<InviteCodes>,
    codes_rx: Receiver<InviteCodes>,

    // Sender/Receiver for async invite codes.
    page_tx: Sender<Page>,
    page_rx: Receiver<Page>,

    // Sender/Receiver for async invite codes.
    otp_tx: Sender<(String, String)>,
    otp_rx: Receiver<(String, String)>,

    // Sender/Receiver for otp code.
    qr_tx: Sender<String>,
    qr_rx: Receiver<String>,

    qr_secret: String,
    qr_code: Option<QrCode>,
    otp_code: String,
    x: Vec<u8>,

    // Silly app state.
    value: u32,
    count: u32,

    codes: Vec<Code>,
    filtered_codes: Vec<Code>,

    search_term: String,
}

impl Default for InviteCodeManager {
    fn default() -> Self {
        let (tx, rx) = std::sync::mpsc::channel();
        let (codes_tx, codes_rx) = std::sync::mpsc::channel();
        let (otp_tx, otp_rx) = std::sync::mpsc::channel();
        let (page_tx, page_rx) = std::sync::mpsc::channel();
        let (qr_tx, qr_rx) = std::sync::mpsc::channel();
        let cookie_store = Arc::new(Jar::default());
        let client = Client::builder()
            .cookie_store(true)
            .cookie_provider(cookie_store.clone())
            .build()
            .unwrap();
        Self {
            page: Page::Login,
            invite_backend: "https://pds.example.com".to_string(),
            username: "".to_string(),
            password: "".to_string(),
            client,
            tx,
            rx,
            codes_rx,
            page_tx,
            codes_tx,
            qr_rx,
            qr_tx,
            value: 1,
            count: 0,
            codes: vec![],
            filtered_codes: vec![],
            search_term: "".to_string(),
            qr_code: None,
            otp_code: "".to_string(),
            x: vec![],
            qr_secret: "".to_string(),
            page_rx,
            otp_tx,
            otp_rx,
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

            let res = self.otp_rx.try_recv();
            if res.is_ok() {
                let res = res.unwrap();
                let totp = TOTP::new(
                    Algorithm::SHA1,
                    6,
                    1,
                    30,
                    Secret::Encoded(res.0).to_bytes().unwrap(),
                    Some("Northsky".to_string()),
                    "Northsky".to_string(),
                )
                .unwrap();
                let qr_code = totp.get_qr_png().unwrap();
                self.qr_code = Some(QrCode {
                    image: qr_code,
                    url: res.1,
                });
                self.page = Page::QrVerify;
            }

            match self.page {
                Page::Home => {
                    let res = self.codes_rx.try_recv();
                    if res.is_ok() {
                        let invite_codes = res.unwrap();
                        self.codes = invite_codes.codes;
                        filter_invites(self);
                    }

                    ui.horizontal(|ui| {
                        if ui.button("Get Invite Codes").clicked() {
                            get_invite_codes(self, self.codes_tx.clone(), ctx.clone());
                        }
                        if ui.text_edit_singleline(&mut self.search_term).changed() {
                            filter_invites(self);
                        }
                        if ui.button("Create Invite Code").clicked() {
                            create_invite_code(self, self.codes_tx.clone(), ctx.clone());
                        }
                    });

                    if !self.filtered_codes.is_empty() {
                        TableBuilder::new(ui)
                            .columns(Column::auto().resizable(true), 6)
                            // .column(Column::remainder())
                            .header(20.0, |mut header| {
                                header.col(|ui| {
                                    ui.heading("Code");
                                });
                                header.col(|ui| {
                                    ui.heading("Used");
                                });
                                header.col(|ui| {
                                    ui.heading("Disabled");
                                });
                                header.col(|ui| {
                                    ui.heading("Used By");
                                });
                                header.col(|ui| {
                                    ui.heading("Used At");
                                });
                                header.col(|ui| {
                                    ui.heading("");
                                });
                            })
                            .body(|mut body| {
                                for code in self.filtered_codes.clone() {
                                    body.row(30.0, |mut row| {
                                        row.col(|ui| {
                                            ui.label(code.code.as_str());
                                        });
                                        row.col(|ui| match code.available > 0 {
                                            true => {
                                                ui.label("y");
                                            }
                                            false => {
                                                ui.label("n");
                                            }
                                        });
                                        row.col(|ui| match code.disabled {
                                            true => {
                                                ui.label("y");
                                            }
                                            false => {
                                                ui.label("n");
                                            }
                                        });
                                        row.col(|ui| {
                                            let binding = code.uses.clone();
                                            if binding.is_empty() {
                                                ui.label("");
                                            } else {
                                                let uses = binding.first().unwrap();
                                                ui.label(uses.used_by.as_str());
                                            }
                                        });
                                        row.col(|ui| {
                                            let binding = code.uses.clone();
                                            if binding.is_empty() {
                                                ui.label("");
                                            } else {
                                                let uses = binding.first().unwrap();
                                                ui.label(uses.used_at.as_str());
                                            }
                                        });
                                        row.col(|ui| {
                                            if ui.button("Disable").clicked() {
                                                disable_invite_code(self, code.code.clone());
                                            }
                                        });
                                    });
                                }
                            });
                    }
                }
                Page::Login => {
                    styles::render_subtitle(ui, "Login");

                    ui.horizontal(|ui| {
                        ui.label("Invite Manager Endpoint:");
                        ui.text_edit_singleline(&mut self.invite_backend);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Username:");
                        ui.text_edit_singleline(&mut self.username);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Password:");
                        ui.add(egui::TextEdit::singleline(&mut self.password).password(true));
                    });
                    if ui.button("Submit").clicked() {
                        login(self, self.page_tx.clone(), self.otp_tx.clone());
                    }
                }
                Page::QrVerify => {
                    ui.add(
                        egui::Image::from_bytes(
                            "bytes://test.png",
                            self.qr_code.clone().unwrap().image.clone(),
                        )
                        .max_height(200f32)
                        .max_width(200f32),
                    );
                    ui.horizontal(|ui| {
                        ui.label("Please enter code:");
                        ui.text_edit_singleline(&mut self.otp_code);
                        if ui.button("Submit").clicked() {
                            verify_otp(self, self.page_tx.clone());
                        }
                    });
                }
                Page::QrValidate => {
                    ui.horizontal(|ui| {
                        ui.label("2fa code:");
                        ui.text_edit_singleline(&mut self.otp_code);
                    });
                    if ui.button("Submit").clicked() {
                        validate_otp(self, self.page_tx.clone());
                    }
                }
            }
        });
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Use {
    #[serde(rename = "usedBy")]
    pub used_by: String,
    #[serde(rename = "usedAt")]
    pub used_at: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Code {
    pub code: String,
    pub available: i32,
    pub disabled: bool,
    #[serde(rename = "forAccount")]
    pub for_account: String,
    #[serde(rename = "createdBy")]
    pub created_by: String,
    #[serde(rename = "createdAt")]
    pub created_at: String,
    pub uses: Vec<Use>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct InviteCodes {
    pub cursor: String,
    pub codes: Vec<Code>,
}

fn get_invite_codes(
    app: &mut InviteCodeManager,
    codes_tx: Sender<InviteCodes>,
    ctx: egui::Context,
) {
    let endpoint = app.invite_backend.clone() + GET_INVITE_CODES;
    let client = app.client.clone();
    tokio::spawn(async move {
        let res = client
            .get(endpoint)
            .header("Content-Type", "application/json")
            .send()
            .await
            .unwrap();
        if !res.status().is_success() {
            panic!("not success")
        }
        let invite_codes = res.json::<InviteCodes>().await;
        match invite_codes {
            Ok(invite_codes) => {
                codes_tx.send(invite_codes).unwrap();
            }
            Err(e) => {
                eprintln!("{}", e);
                panic!("Invite Codes")
            }
        }

        ctx.request_repaint();
    });
}

#[derive(Serialize, Deserialize)]
struct CreateInviteCodeBody {
    #[serde(rename = "codeCount")]
    pub code_count: i32,
    #[serde(rename = "useCount")]
    pub use_count: i32,
}

fn create_invite_code(
    app: &mut InviteCodeManager,
    codes_tx: Sender<InviteCodes>,
    ctx: egui::Context,
) {
    let endpoint = app.invite_backend.clone() + CREATE_INVITE_CODES;
    let client = app.client.clone();
    let body = CreateInviteCodeBody {
        code_count: 1,
        use_count: 1,
    };
    tokio::spawn(async move {
        let res = client
            .post(endpoint)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .unwrap();
        if !res.status().is_success() {
            panic!("not success")
        }
        ctx.request_repaint();
    });
}

fn filter_invites(app: &mut InviteCodeManager) {
    app.filtered_codes.clear();
    for code in app.codes.clone() {
        if code.code.contains(app.search_term.as_str()) {
            app.filtered_codes.push(code.clone());
        }
    }
}

#[derive(Serialize, Deserialize)]
struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize)]
struct GenerateOtpResponse {
    pub base32: String,
    pub otpauth_url: String,
}

fn login(app: &mut InviteCodeManager, page_tx: Sender<Page>, otp_tx: Sender<(String, String)>) {
    let login_endpoint = app.invite_backend.clone() + LOGIN;
    let otp_generate_endpoint = app.invite_backend.clone() + GENERATE_OTP;
    let client = app.client.clone();
    let username = app.username.clone();
    let password = app.password.clone();
    tokio::spawn(async move {
        let res = client
            .post(login_endpoint)
            .header("Content-Type", "application/json")
            .json(&LoginRequest { username, password })
            .send()
            .await
            .unwrap();
        if !res.status().is_success() {
            panic!("not success")
        }

        if res.status() == StatusCode::OK {
            page_tx.send(Page::QrValidate).unwrap();
        }

        if res.status() == StatusCode::from_u16(201).unwrap() {
            let res = client
                .post(otp_generate_endpoint)
                .header("Content-Type", "application/json")
                .send()
                .await
                .unwrap();
            if !res.status().is_success() {
                panic!("not success")
            }
            let res = res.json::<GenerateOtpResponse>().await.unwrap();
            otp_tx.send((res.base32, res.otpauth_url)).unwrap();
        }
    });
}

#[derive(Serialize, Deserialize)]
struct VerifyRequest {
    pub token: String,
}

fn verify_otp(app: &mut InviteCodeManager, page_tx: Sender<Page>) {
    let endpoint = app.invite_backend.clone() + VERIFY_OTP;
    let client = app.client.clone();
    let token = app.otp_code.clone();
    tokio::spawn(async move {
        let res = client
            .post(endpoint)
            .header("Content-Type", "application/json")
            .json(&VerifyRequest { token })
            .send()
            .await
            .unwrap();
        if !res.status().is_success() {
            panic!("not success")
        }
        page_tx.send(Page::Home).unwrap();
    });
}

#[derive(Serialize, Deserialize)]
struct ValidateRequest {
    pub token: String,
}

fn validate_otp(app: &mut InviteCodeManager, page_tx: Sender<Page>) {
    let endpoint = app.invite_backend.clone() + VALIDATE_OTP;
    let client = app.client.clone();
    let token = app.otp_code.clone();
    tokio::spawn(async move {
        let res = client
            .post(endpoint)
            .header("Content-Type", "application/json")
            .json(&ValidateRequest { token })
            .send()
            .await
            .unwrap();
        if !res.status().is_success() {
            panic!("not success")
        }
        page_tx.send(Page::Home).unwrap();
    });
}

#[derive(Serialize, Deserialize)]
struct DisableInviteCodesRequest {
    pub codes: Vec<String>,
    pub accounts: Vec<String>,
}

fn disable_invite_code(app: &mut InviteCodeManager, code: String) {
    let endpoint = app.invite_backend.clone() + DISABLE_INVITE_CODES;
    let client = app.client.clone();
    let body = DisableInviteCodesRequest {
        codes: vec![code],
        accounts: vec![],
    };
    tokio::spawn(async move {
        let res = client
            .post(endpoint)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .unwrap();
        if !res.status().is_success() {
            panic!("not success")
        }
    });
}
