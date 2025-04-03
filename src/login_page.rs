use crate::qr_validate_page::QrValidatePage;
use crate::qr_verify_page::QrVerifyPage;
use crate::{LOGIN, Page};
use egui::Ui;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::sync::mpsc::{Receiver, Sender};

use crate::styles;

pub struct LoginPage {
    invite_backend: String,
    username: String,
    password: String,
    page_tx: Sender<Page>,
    client: Client,
    error_tx: Sender<String>,
    error_rx: Receiver<String>,
    error_message: String,
}

impl LoginPage {
    pub fn new(page_tx: Sender<Page>, client: Client, invite_backend: String) -> Self {
        // For error message handling
        let (error_tx, error_rx) = std::sync::mpsc::channel();

        Self {
            invite_backend,
            username: "".to_string(),
            password: "".to_string(),
            page_tx,
            client,
            error_tx,
            error_rx,
            error_message: "".to_string(),
        }
    }

    pub fn show(&mut self, ui: &mut Ui) {
        styles::render_subtitle(ui, "Login!");

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
            );
            styles::render_input(ui, "Username", &mut self.username, false);
            styles::render_input(ui, "Password", &mut self.password, true);
        });

        // Display current error message, if exists
        if !self.error_message.is_empty() {
            styles::render_error(ui, &self.error_message);
        }

        styles::render_button(ui, "Submit", || {
            self.login();
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
        let invite_backend = self.invite_backend.clone();

        tokio::spawn(async move {
            let res = match client
                .post(login_endpoint)
                .header("Content-Type", "application/json")
                .json(&LoginRequest { username, password })
                .send()
                .await
            {
                Ok(response) => response,
                Err(err) => {
                    eprintln!("Login error: {}", err);
                    error_tx.send("An error occured connecting to the server. Please check the invite manager endpoint and try again.".to_string()).unwrap();
                    return;
                }
            };

            if !res.status().is_success() {
                error_tx
                    .send("Login failed. Please check your credentials and try again.".to_string())
                    .unwrap();
                return;
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
