#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use egui::Widget;
use egui_extras::{Column, TableBuilder};
use serde::{Deserialize, Serialize};
use std::sync::mpsc::{Receiver, Sender};
use std::time::Duration;
use tokio::runtime::Runtime;
use totp_rs::{Algorithm, Secret, TOTP};

#[derive(PartialEq, Eq, Default)]
pub struct QrCode {
    image: Vec<u8>,
    url: String,
}

const GET_INVITE_CODES: &str = "/xrpc/com.atproto.admin.getInviteCodes";
const DISABLE_INVITE_CODES: &str = "/xrpc/com.atproto.admin.disableInviteCodes";
const CREATE_INVITE_CODE: &str = "/xrpc/com.atproto.admin.server.createInviteCode";
const CREATE_INVITE_CODES: &str = "/xrpc/com.atproto.admin.server.createInviteCodes";


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
        Box::new(|cc| Ok(Box::<InviteCodeManager>::default())),
    )
}

struct InviteCodeManager {
    // Sender/Receiver for async notifications.
    tx: Sender<u32>,
    rx: Receiver<u32>,

    // Sender/Receiver for async invite codes.
    codes_tx: Sender<InviteCodes>,
    codes_rx: Receiver<InviteCodes>,

    qr_code: Option<QrCode>,

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
        Self {
            tx,
            rx,
            codes_rx,
            codes_tx,
            value: 1,
            count: 0,
            codes: vec![],
            filtered_codes: vec![],
            search_term: "".to_string(),
            qr_code: None,
        }
    }
}

impl eframe::App for InviteCodeManager {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            match &self.qr_code {
                None => {}
                Some(qr_code) => {
                    let x = egui::Image::from_bytes(
                        "bytes://".to_string() + "test",
                        qr_code.image.clone(),
                    );
                    egui::Image::ui.add(x);
                }
            }
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
                    let totp = TOTP::new(
                        Algorithm::SHA1,
                        6,
                        1,
                        30,
                        Secret::Encoded("KRSXG5CTMVRXEZLUKN2XAZLSKNSWG4TFOQ".to_string())
                            .to_bytes()
                            .unwrap(),
                        Some("Northsky".to_string()),
                        "Northsky".to_string(),
                    )
                    .unwrap();
                    let qr_code = totp.get_qr_png().unwrap();
                    self.qr_code = Some(QrCode {
                        image: qr_code,
                        url: "bytes://northskyqr".to_string(),
                    });
                    // get_invite_codes(self, self.codes_tx.clone(), ctx.clone());
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
                        for code in &self.filtered_codes {
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
                                        let uses = binding.get(0).clone().unwrap();
                                        ui.label(uses.usedBy.as_str());
                                    }
                                });
                                row.col(|ui| {
                                    let binding = code.uses.clone();
                                    if binding.is_empty() {
                                        ui.label("");
                                    } else {
                                        let uses = binding.get(0).clone().unwrap();
                                        ui.label(uses.usedAt.as_str());
                                    }
                                });
                                row.col(|ui| if ui.button("Disable").clicked() {});
                            });
                        }
                    });
            }
        });
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Use {
    pub usedBy: String,
    pub usedAt: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct Code {
    pub code: String,
    pub available: i32,
    pub disabled: bool,
    pub forAccount: String,
    pub createdBy: String,
    pub createdAt: String,
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
    tokio::spawn(async move {
        let client = reqwest::Client::new();
        let res = client
            .get("https://pds.ripperoni.com".to_string() + GET_INVITE_CODES)
            .header("Content-Type", "application/json")
            .header(
                "Authorization",
                "Basic dnlndlanwdln=",
            )
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

fn create_invite_code(
    app: &mut InviteCodeManager,
    codes_tx: Sender<InviteCodes>,
    ctx: egui::Context,
) {
    tokio::spawn(async move {
        let client = reqwest::Client::new();
        let res = client
            .get("https://pds.ripperoni.com".to_string() + GET_INVITE_CODES)
            .header("Content-Type", "application/json")
            .header(
                "Authorization",
                "Basic dnlndlanwdln=",
            )
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

fn filter_invites(app: &mut InviteCodeManager) {
    app.filtered_codes.clear();
    for code in app.codes.clone() {
        if code.code.contains(app.search_term.as_str()) {
            app.filtered_codes.push(code.clone());
        }
    }
}

fn generate_qr_code() {}
