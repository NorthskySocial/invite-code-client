use crate::util::create_task;
use crate::{
    CREATE_INVITE_CODES, DISABLE_INVITE_CODES, GENERATE_OTP, GET_INVITE_CODES, LOGIN, Page,
    VALIDATE_OTP, VERIFY_OTP, styles,
};
use eframe::egui;
use eframe::egui::{Context, Image, Ui};
use egui_extras::{Column, Size, StripBuilder, TableBuilder};
#[cfg(not(target_arch = "wasm32"))]
use reqwest::cookie::Jar;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
#[cfg(not(target_arch = "wasm32"))]
use std::sync::Arc;
use std::sync::mpsc::{Receiver, Sender};
use totp_rs::{Algorithm, Secret, TOTP};

pub struct InviteCodeManager {
    page: Page,
    page_tx: Sender<Page>,
    page_rx: Receiver<Page>,
    error_rx: Receiver<String>,

    codes: Vec<Code>,
    filtered_codes: Vec<Code>,
    search_term: String,
    invite_backend: String,
    client: Client,

    // Sender/Receiver for invite codes
    invite_code_tx: Sender<InviteCodes>,
    invite_code_rx: Receiver<InviteCodes>,
    qr_code: Option<QrCodeBase>,
    error_tx: Sender<String>,
    otp_code: String,
    error_message: String,
    qr_tx: Sender<(String, String)>,
    qr_rx: Receiver<(String, String)>,
    username: String,
    password: String,
    generated_otp: bool,
}

impl Default for InviteCodeManager {
    #[cfg(not(target_arch = "wasm32"))]
    fn default() -> Self {
        let (page_tx, page_rx) = std::sync::mpsc::channel();
        let (error_tx, error_rx) = std::sync::mpsc::channel();
        let (invite_code_tx, invite_code_rx) = std::sync::mpsc::channel();
        let (qr_tx, qr_rx) = std::sync::mpsc::channel();
        let cookie_store = Arc::new(Jar::default());
        let client = Client::builder()
            .cookie_store(true)
            .cookie_provider(cookie_store.clone())
            .build()
            .unwrap();
        Self {
            page: Page::Login,
            page_tx,
            page_rx,
            error_rx,
            codes: vec![],
            filtered_codes: vec![],
            search_term: "".to_string(),
            invite_backend: "https://invites.northsky.social".to_string(),
            client,
            invite_code_tx,
            invite_code_rx,
            qr_code: None,
            error_tx,
            otp_code: "".to_string(),
            error_message: "".to_string(),
            qr_tx,
            qr_rx,
            username: "".to_string(),
            password: "".to_string(),
            generated_otp: false,
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn default() -> Self {
        let (page_tx, page_rx) = std::sync::mpsc::channel();
        let (error_tx, error_rx) = std::sync::mpsc::channel();
        let (invite_code_tx, invite_code_rx) = std::sync::mpsc::channel();
        let (qr_tx, qr_rx) = std::sync::mpsc::channel();
        let client = Client::builder().build().unwrap();
        Self {
            page: Page::Login,
            page_tx,
            page_rx,
            error_rx,
            codes: vec![],
            filtered_codes: vec![],
            search_term: "".to_string(),
            invite_backend: "https://invites.northsky.social".to_string(),
            client,
            invite_code_tx,
            invite_code_rx,
            qr_code: None,
            error_tx,
            otp_code: "".to_string(),
            error_message: "".to_string(),
            qr_tx,
            qr_rx,
            username: "".to_string(),
            password: "".to_string(),
        }
    }
}

impl eframe::App for InviteCodeManager {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Basic window styling.
            styles::set_text_color(ui);
            styles::render_title(ui, ctx, styles::FRAME_TITLE);

            let res = self.page_rx.try_recv();
            if res.is_ok() {
                self.page = res.unwrap();
            }

            match &mut self.page {
                Page::Home => {
                    self.show_home(ui);
                }
                Page::Login => {
                    self.show_login(ui, ctx);
                }
                Page::QrVerify => {
                    self.show_verify_qr(ui, ctx);
                }
                Page::QrValidate => {
                    self.show_validate_qr(ui, ctx);
                }
            }
        });
        egui::TopBottomPanel::bottom("bottom_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Light Mode").clicked() {
                    ctx.set_theme(egui::Theme::Light);
                }
                ui.add_space(10.0);
                if ui.button("Dark Mode").clicked() {
                    ctx.set_theme(egui::Theme::Dark);
                }
            });
        });
    }
}

impl InviteCodeManager {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        styles::setup_fonts(&cc.egui_ctx);
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        // if let Some(storage) = cc.storage {
        //     eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        // } else {
        //     Default::default()
        // }
        InviteCodeManager::default()
    }

    fn invite_table_ui(&mut self, ui: &mut Ui) {
        let available_height = ui.available_height();
        let table = TableBuilder::new(ui)
            .striped(true)
            .resizable(true)
            .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
            .columns(Column::auto().resizable(true), 6)
            .min_scrolled_height(0.0)
            .max_scroll_height(available_height);
        table
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
                                self.disable_invite_code(code.code.clone());
                            }
                        });
                    });
                }
            });
    }

    pub fn show_home(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            // Check for new error messages
            if let Ok(error_message) = self.error_rx.try_recv() {
                self.error_message = error_message;
            }

            // Display current error message, if exists
            if !self.error_message.is_empty() {
                styles::render_error(ui, &self.error_message);
            }

            if let Ok(invite_codes) = self.invite_code_rx.try_recv() {
                self.codes = invite_codes.codes;
                self.filter_invites();
            }

            ui.horizontal(|ui| {
                styles::render_unaligned_button(ui, "Get Invite Codes", || {
                    self.get_invite_codes();
                });
                ui.label("Filter:");
                if ui
                    .add(styles::render_base_input(
                        &mut self.search_term,
                        false,
                        true,
                    ))
                    .changed()
                {
                    self.filter_invites();
                }
                styles::render_unaligned_button(ui, "Create Invite Code", || {
                    self.create_invite_code();
                    self.get_invite_codes();
                });
            });
            ui.separator();
            StripBuilder::new(ui)
                .size(Size::remainder().at_least(100.0))
                .vertical(|mut strip| {
                    strip.cell(|ui| {
                        egui::ScrollArea::both().show(ui, |ui| self.invite_table_ui(ui));
                    });
                });
        });
    }

    fn get_invite_codes(&mut self) {
        let endpoint = self.invite_backend.clone() + GET_INVITE_CODES;
        let client = self.client.clone();
        let invite_code_tx = self.invite_code_tx.clone();
        let error_tx = self.error_tx.clone();

        create_task(async move {
            let res = client
                .get(endpoint)
                .header("Content-Type", "application/json")
                .send()
                .await
                .unwrap();
            if !res.status().is_success() {
                let status = res.status();
                error_tx
                    .send(format!("Received HTTP Status: {status}"))
                    .unwrap();
                return;
            }
            let invite_codes = res.json::<InviteCodes>().await;
            match invite_codes {
                Ok(invite_codes) => {
                    invite_code_tx.send(invite_codes).unwrap();
                }
                Err(e) => {
                    error_tx.send(format!("Error: {e}")).unwrap();
                }
            }
        });
    }

    fn filter_invites(&mut self) {
        self.filtered_codes.clear();
        for code in self.codes.clone() {
            if code.code.contains(self.search_term.as_str()) {
                self.filtered_codes.push(code.clone());
            }
        }
    }

    fn disable_invite_code(&mut self, code: String) {
        let endpoint = self.invite_backend.clone() + DISABLE_INVITE_CODES;
        let client = self.client.clone();
        let error_tx = self.error_tx.clone();
        let body = DisableInviteCodesRequest {
            codes: vec![code],
            accounts: vec![],
        };

        create_task(async move {
            let res = client
                .post(endpoint)
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await
                .unwrap();
            if !res.status().is_success() {
                let status = res.status();
                error_tx
                    .send(format!("Received HTTP Status: {status}"))
                    .unwrap();
            }
        });
    }

    fn create_invite_code(&mut self) {
        let endpoint = self.invite_backend.clone() + CREATE_INVITE_CODES;
        let client = self.client.clone();
        let error_tx = self.error_tx.clone();
        let body = CreateInviteCodeBody {
            code_count: 1,
            use_count: 1,
        };

        create_task(async move {
            let res = client
                .post(endpoint)
                .header("Content-Type", "application/json")
                .json(&body)
                .send()
                .await
                .unwrap();
            if !res.status().is_success() {
                let status = res.status();
                error_tx
                    .send(format!("Received HTTP Status: {status}"))
                    .unwrap();
            }
        });
    }

    pub fn show_validate_qr(&mut self, ui: &mut Ui, ctx: &Context) {
        ui.vertical_centered(|ui| {
            styles::render_subtitle(ui, ctx, "Two Factor Authentication!");

            // Check for new error messages
            if let Ok(error_message) = self.error_rx.try_recv() {
                self.error_message = error_message;
            }

            styles::render_input(ui, "2FA code", &mut self.otp_code, false, None);

            // Display current error message, if exists
            if !self.error_message.is_empty() {
                styles::render_error(ui, &self.error_message);
            }

            styles::render_button(ui, ctx, "Submit", || {
                self.validate_otp();
            });
        });
    }

    fn validate_otp(&mut self) {
        let error_tx = self.error_tx.clone();

        // Clearing error messages
        error_tx.send("".to_string()).unwrap();

        // Validation
        if self.otp_code.is_empty() {
            error_tx
                .send("Please enter a 2FA code.".to_string())
                .unwrap();
            return;
        }

        let endpoint = self.invite_backend.clone() + VALIDATE_OTP;
        let token = self.otp_code.clone();
        let page_tx = self.page_tx.clone();
        let client = self.client.clone();

        create_task(async move {
            let res = client
                .post(endpoint)
                .header("Content-Type", "application/json")
                .json(&ValidateRequest { token })
                .send()
                .await
                .unwrap();
            if !res.status().is_success() {
                error_tx.send("There was an error validating your 2FA code. Please check your code and try again.".to_string()).unwrap();
                return;
            }
            page_tx.send(Page::Home).unwrap();
        });
    }

    fn generate_otp(&mut self) {
        let otp_generate_endpoint = self.invite_backend.clone() + GENERATE_OTP;
        let client = self.client.clone();
        let qr_tx = self.qr_tx.clone();
        let error_tx = self.error_tx.clone();

        create_task(async move {
            let res = client
                .post(otp_generate_endpoint)
                .header("Content-Type", "application/json")
                .send()
                .await
                .unwrap();
            if !res.status().is_success() {
                let status = res.status();
                error_tx
                    .send(format!("Received HTTP Status: {status}"))
                    .unwrap();
                return;
            }
            let res = res.json::<GenerateOtpResponse>().await.unwrap();
            qr_tx.send((res.base32, res.otpauth_url)).unwrap();
        });
    }

    pub fn show_verify_qr(&mut self, ui: &mut Ui, ctx: &Context) {
        if !self.generated_otp {
            self.generated_otp = true;
            self.generate_otp();
        }

        let res = self.qr_rx.try_recv();
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
            self.qr_code = Some(QrCodeBase {
                image: qr_code,
                url: res.1,
            });
        }

        styles::render_subtitle(ui, ctx, "Two Factor Authentication Setup!");

        // Check for new error messages
        if let Ok(error_message) = self.error_rx.try_recv() {
            self.error_message = error_message;
        }

        ui.vertical_centered(|ui| {
            ui.label("Scan the QR code with your 2FA app");

            match self.qr_code.clone() {
                Some(qr_code) => {
                    ui.add(
                        Image::from_bytes("bytes://test.png", qr_code.image.clone())
                            .max_height(200f32)
                            .max_width(200f32),
                    );
                }
                None => {
                    ui.label("Generating QR code...");
                }
            }

            styles::render_input(
                ui,
                "To confirm, enter a generated 2FA code",
                &mut self.otp_code,
                false,
                None,
            );
            // Display current error message, if exists
            if !self.error_message.is_empty() {
                styles::render_error(ui, &self.error_message);
            }

            styles::render_button(ui, ctx, "Submit", || {
                self.verify_otp();
            });
        });
    }

    fn verify_otp(&mut self) {
        let error_tx = self.error_tx.clone();

        // Clearing error messages
        error_tx.send("".to_string()).unwrap();

        // Validation
        if self.otp_code.is_empty() {
            error_tx
                .send("Please enter a 2FA code.".to_string())
                .unwrap();
            return;
        }

        let endpoint = self.invite_backend.clone() + VERIFY_OTP;
        let client = self.client.clone();
        let token = self.otp_code.clone();
        let page_tx = self.page_tx.clone();
        let error_tx = self.error_tx.clone();

        create_task(async move {
            let res = client
                .post(endpoint)
                .header("Content-Type", "application/json")
                .json(&VerifyRequest { token })
                .send()
                .await
                .unwrap();
            if !res.status().is_success() {
                let status = res.status();
                error_tx
                    .send(format!("Received HTTP Status: {status}"))
                    .unwrap();
                return;
            }
            page_tx.send(Page::Home).unwrap();
        });
    }

    pub fn show_login(&mut self, ui: &mut Ui, ctx: &Context) {
        styles::render_subtitle(ui, ctx, "Login!");

        // Check for new error messages
        if let Ok(error_message) = self.error_rx.try_recv() {
            self.error_message = error_message;
        }

        ui.vertical_centered(|ui| {
            styles::render_input(
                ui,
                "Invite Manager Endpoint",
                &mut self.invite_backend,
                false,
                None,
            );
            styles::render_input(ui, "Username", &mut self.username, false, None);
            styles::render_input(ui, "Password", &mut self.password, true, None);
            // Display current error message, if exists
            if !self.error_message.is_empty() {
                styles::render_error(ui, &self.error_message);
            }

            styles::render_button(ui, ctx, "Submit", || {
                self.login();
            });
        });
    }

    fn login(&mut self) {
        let error_tx = self.error_tx.clone();

        // Clearing error messages
        error_tx.send("".to_string()).unwrap();

        // Validation
        if self.invite_backend.is_empty() {
            self.error_tx
                .send("Please enter a valid invite manager endpoint.".to_string())
                .unwrap();
            return;
        }

        if self.username.is_empty() {
            self.error_tx
                .send("Please enter a valid username.".to_string())
                .unwrap();
            return;
        }

        if self.password.is_empty() {
            self.error_tx
                .send("Please enter a valid password.".to_string())
                .unwrap();
            return;
        }

        let login_endpoint = self.invite_backend.clone() + LOGIN;

        let client = self.client.clone();
        let username = self.username.clone();
        let password = self.password.clone();
        let page_tx = self.page_tx.clone();
        let error_tx = self.error_tx.clone();

        create_task(async move {
            let res = match client
                .post(login_endpoint)
                .header("Content-Type", "application/json")
                .json(&LoginRequest { username, password })
                .send()
                .await
            {
                Ok(response) => response,
                Err(err) => {
                    error_tx.send(format!("Error: {err}")).unwrap();
                    return;
                }
            };

            if !res.status().is_success() {
                let status = res.status();
                error_tx
                    .send(format!("Received HTTP Status: {status}"))
                    .unwrap();
                return;
            }

            if res.status() == StatusCode::OK {
                page_tx.send(Page::QrValidate).unwrap();
            } else if res.status() == StatusCode::from_u16(201).unwrap() {
                page_tx.send(Page::QrVerify).unwrap();
            }
        });
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct InviteCodes {
    pub cursor: String,
    pub codes: Vec<Code>,
}

#[derive(Serialize, Deserialize)]
struct CreateInviteCodeBody {
    #[serde(rename = "codeCount")]
    pub code_count: i32,
    #[serde(rename = "useCount")]
    pub use_count: i32,
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

#[derive(Serialize, Deserialize)]
struct DisableInviteCodesRequest {
    pub codes: Vec<String>,
    pub accounts: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct ValidateRequest {
    pub token: String,
}

#[derive(Serialize, Deserialize)]
struct VerifyRequest {
    pub token: String,
}

#[derive(PartialEq, Eq, Default, Clone)]
pub struct QrCodeBase {
    image: Vec<u8>,
    url: String,
}

#[derive(Serialize, Deserialize)]
struct GenerateOtpResponse {
    pub base32: String,
    pub otpauth_url: String,
}

#[derive(Serialize, Deserialize)]
struct LoginRequest {
    pub username: String,
    pub password: String,
}
