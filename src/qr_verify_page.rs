use crate::home_page::HomePage;
use crate::{GENERATE_OTP, Page, VERIFY_OTP};
use egui::Ui;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::mpsc::{Receiver, Sender};
use totp_rs::{Algorithm, Secret, TOTP};

use crate::styles;

#[derive(Serialize, Deserialize)]
struct GenerateOtpResponse {
    pub base32: String,
    pub otpauth_url: String,
}

pub struct QrVerifyPage {
    client: Client,
    invite_backend: String,
    otp_code: String,
    page_tx: Sender<Page>,
    qr_tx: Sender<(String, String)>,
    qr_rx: Receiver<(String, String)>,
    qr_code: Option<QrCodeBase>,
    error_tx: Sender<String>,
    error_rx: Receiver<String>,
    error_message: String,
}

impl QrVerifyPage {
    pub fn new(client: Client, page_tx: Sender<Page>, invite_backend: String) -> Self {
        let (qr_tx, qr_rx) = std::sync::mpsc::channel();
        let (error_tx, error_rx) = std::sync::mpsc::channel();

        let mut page = QrVerifyPage {
            client,
            invite_backend,
            otp_code: "".to_string(),
            page_tx,
            qr_tx,
            qr_rx,
            qr_code: None,
            error_tx,
            error_rx,
            error_message: "".to_string(),
        };
        page.generate_otp();
        page
    }

    fn generate_otp(&mut self) {
        let otp_generate_endpoint = self.invite_backend.clone() + GENERATE_OTP;
        let client = self.client.clone();
        let qr_tx = self.qr_tx.clone();
        tokio::spawn(async move {
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
            qr_tx.send((res.base32, res.otpauth_url)).unwrap();
        });
    }

    pub fn show(&mut self, ui: &mut Ui) {
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

        styles::render_subtitle(ui, "Two Factor Authentication Setup!");

        // Check for new error messages
        if let Ok(error_message) = self.error_rx.try_recv() {
            self.error_message = error_message;
        }

        ui.vertical_centered(|ui| {
            ui.label("Scan the QR code with your 2FA app");

            ui.add(
                egui::Image::from_bytes(
                    "bytes://test.png",
                    self.qr_code.clone().unwrap().image.clone(),
                )
                .max_height(200f32)
                .max_width(200f32),
            );

            styles::render_input(
                ui,
                "To confirm, enter a generated 2FA code",
                &mut self.otp_code,
                false,
            );
        });

        // Display current error message, if exists
        if !self.error_message.is_empty() {
            styles::render_error(ui, &self.error_message);
        }

        styles::render_button(ui, "Submit", || {
            self.verify_otp();
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
        let invite_backend = self.invite_backend.clone();
        tokio::spawn(async move {
            let res = client
                .post(endpoint)
                .header("Content-Type", "application/json")
                .json(&VerifyRequest { token })
                .send()
                .await
                .unwrap();
            if !res.status().is_success() {
                error_tx
                    .send(
                        "An error occured trying to verify your code. Please, try again."
                            .to_string(),
                    )
                    .unwrap();
                eprintln!("Verify OTP not successful");
                return;
            }
            page_tx
                .send(Page::Home(HomePage::new(client, invite_backend)))
                .unwrap();
        });
    }
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
