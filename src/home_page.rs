use crate::{CREATE_INVITE_CODES, DISABLE_INVITE_CODES, GET_INVITE_CODES, styles};
use egui::Ui;
use egui_extras::{Column, TableBuilder};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::mpsc::{Receiver, Sender};

#[derive(Serialize, Deserialize)]
struct DisableInviteCodesRequest {
    pub codes: Vec<String>,
    pub accounts: Vec<String>,
}

pub struct HomePage {
    codes: Vec<Code>,
    filtered_codes: Vec<Code>,
    search_term: String,
    invite_backend: String,
    client: Client,

    // Sender/Receiver for invite codes
    invite_code_tx: Sender<InviteCodes>,
    invite_code_rx: Receiver<InviteCodes>,
}

impl HomePage {
    pub fn new(client: Client, invite_backend: String) -> Self {
        let (invite_code_tx, invite_code_rx) = std::sync::mpsc::channel();
        Self {
            codes: vec![],
            filtered_codes: vec![],
            search_term: "".to_string(),
            invite_backend,
            client,
            invite_code_tx,
            invite_code_rx,
        }
    }

    pub fn show(&mut self, ui: &mut Ui) {
        let res = self.invite_code_rx.try_recv();
        if res.is_ok() {
            let invite_codes = res.unwrap();
            self.codes = invite_codes.codes;
            self.filter_invites();
        }

        ui.horizontal(|ui| {
            styles::render_unaligned_button(ui, "Get Invite Codes", || {
                self.get_invite_codes();
            });
            ui.vertical(|ui| {
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
            });
            styles::render_unaligned_button(ui, "Create Invite Code", || {
                self.create_invite_code();
            });
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
                                    self.disable_invite_code(code.code.clone());
                                }
                            });
                        });
                    }
                });
        }
    }

    fn get_invite_codes(&mut self) {
        let endpoint = self.invite_backend.clone() + GET_INVITE_CODES;
        let client = self.client.clone();
        let invite_code_tx = self.invite_code_tx.clone();
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
                    invite_code_tx.send(invite_codes).unwrap();
                }
                Err(e) => {
                    eprintln!("{}", e);
                    panic!("Invite Codes")
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

    fn create_invite_code(&mut self) {
        let endpoint = self.invite_backend.clone() + CREATE_INVITE_CODES;
        let client = self.client.clone();
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
