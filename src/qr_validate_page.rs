use crate::home_page::HomePage;
use crate::{Page, VALIDATE_OTP};
use egui::Ui;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::mpsc::{Sender, Receiver};

use crate::styles;

pub struct QrValidatePage {
    invite_backend: String,
    otp_code: String,
    client: Client,
    page_tx: Sender<Page>,
    error_tx: Sender<String>,
    error_rx: Receiver<String>,
    error_message: String,
}

impl QrValidatePage {
    pub fn new(client: Client, page_tx: Sender<Page>, invite_backend: String) -> Self {
        let (error_tx, error_rx) = std::sync::mpsc::channel();

        Self {
            invite_backend,
            otp_code: "".to_string(),
            client,
            page_tx,
            error_tx,
            error_rx,
            error_message: "".to_string(),
        }
    }

    pub fn show(&mut self, ui: &mut Ui) {
        styles::render_subtitle(ui, "Two Factor Authentication!");

        // Check for new error messages
        if let Ok(error_message) = self.error_rx.try_recv() {
            self.error_message = error_message;
        }

        styles::render_input(ui, "2FA code", &mut self.otp_code, false);

        // Display current error message, if exists
        if self.error_message != "" {
            styles::render_error(ui, &self.error_message);
        }

        styles::render_button(ui, "Submit", || {
            self.validate_otp();
        });
    }

    fn validate_otp(&mut self) {
        let error_tx = self.error_tx.clone();

        // Clearing error messages
        error_tx.send("".to_string()).unwrap();

        // Validation
        if self.otp_code.is_empty() {
            error_tx.send("Please enter a 2FA code.".to_string()).unwrap();
            return;
        }

        let endpoint = self.invite_backend.clone() + VALIDATE_OTP;
        let token = self.otp_code.clone();
        let page_tx = self.page_tx.clone();
        let client = self.client.clone();
        let invite_backend = self.invite_backend.clone();

        tokio::spawn(async move {
            let res = client
                .post(endpoint)
                .header("Content-Type", "application/json")
                .json(&ValidateRequest { token })
                .send()
                .await
                .unwrap();
            if !res.status().is_success() {
                eprintln!("QR validate error");
                error_tx.send("There was an error validating your 2FA code. Please check your code and try again.".to_string()).unwrap();
                return;
            }
            page_tx
                .send(Page::Home(HomePage::new(client, invite_backend)))
                .unwrap();
        });
    }
}

#[derive(Serialize, Deserialize)]
struct ValidateRequest {
    pub token: String,
}
