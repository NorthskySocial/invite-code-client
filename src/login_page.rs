use crate::qr_validate_page::QrValidatePage;
use crate::qr_verify_page::QrVerifyPage;
use crate::{LOGIN, Page};
use egui::Ui;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::sync::mpsc::Sender;

use crate::styles;

pub struct LoginPage {
    invite_backend: String,
    username: String,
    password: String,
    page_tx: Sender<Page>,
    client: Client,
}

impl LoginPage {
    pub fn new(page_tx: Sender<Page>, client: Client, invite_backend: String) -> Self {
        Self {
            invite_backend,
            username: "".to_string(),
            password: "".to_string(),
            page_tx,
            client,
        }
    }

    pub fn show(&mut self, ui: &mut Ui) {
        styles::render_subtitle(ui, "Login!");

        ui.vertical_centered(|ui| {
            styles::render_input(ui, "Invite Manager Endpoint", &mut self.invite_backend, false);
            styles::render_input(ui, "Username", &mut self.username, false);
            styles::render_input(ui, "Password", &mut self.password, true);
        });

        styles::render_button(ui, "Submit", || {
            self.login();
        });
    }

    fn login(&mut self) {
        let login_endpoint = self.invite_backend.clone() + LOGIN;

        let client = self.client.clone();
        let username = self.username.clone();
        let password = self.password.clone();
        let page_tx = self.page_tx.clone();
        let invite_backend = self.invite_backend.clone();
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
                page_tx
                    .send(Page::QrValidate(QrValidatePage::new(
                        client,
                        page_tx.clone(),
                        invite_backend,
                    )))
                    .unwrap();
            } else if res.status() == StatusCode::from_u16(201).unwrap() {
                page_tx
                    .send(Page::QrVerify(QrVerifyPage::new(
                        client,
                        page_tx.clone(),
                        invite_backend,
                    )))
                    .unwrap();
            }
        });
    }
}

#[derive(Serialize, Deserialize)]
struct LoginRequest {
    pub username: String,
    pub password: String,
}
