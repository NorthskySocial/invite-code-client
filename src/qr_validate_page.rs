use crate::home_page::HomePage;
use crate::{Page, VALIDATE_OTP};
use egui::Ui;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::mpsc::Sender;

pub struct QrValidatePage {
    invite_backend: String,
    otp_code: String,
    client: Client,
    page_tx: Sender<Page>,
}

impl QrValidatePage {
    pub fn new(client: Client, page_tx: Sender<Page>, invite_backend: String) -> Self {
        Self {
            invite_backend,
            otp_code: "".to_string(),
            client,
            page_tx,
        }
    }

    pub fn show(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label("2FA code:");
            ui.text_edit_singleline(&mut self.otp_code);
        });
        if ui.button("Submit").clicked() {
            self.validate_otp();
        }
    }

    fn validate_otp(&mut self) {
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
                panic!("not success")
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
