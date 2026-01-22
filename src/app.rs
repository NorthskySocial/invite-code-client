use crate::util::create_task;
use crate::{
    CREATE_INVITE_CODES, DISABLE_INVITE_CODES, FilterStatus, GENERATE_OTP, GET_INVITE_CODES, LOGIN,
    Page, VALIDATE_OTP, VERIFY_OTP, styles,
};
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::{format, vec};
use chrono::{DateTime, Utc};
use core::clone::Clone;
use core::default::Default;
use core::option::Option;
#[cfg(target_arch = "wasm32")]
use crossbeam_channel::{Receiver, Sender};
use eframe::egui;
use eframe::egui::{Context, Image, RichText, Ui};
use egui_extras::{Column, TableBuilder};
#[cfg(not(target_arch = "wasm32"))]
use reqwest::cookie::Jar;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::Arc;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::mpsc::{Receiver, Sender};
use totp_rs::{Algorithm, Secret, TOTP};

#[cfg(target_arch = "wasm32")]
async fn read_clipboard_web() -> Option<String> {
    use wasm_bindgen::prelude::*;
    use wasm_bindgen_futures::JsFuture;

    let window = web_sys::window()?;
    let navigator = window.navigator();
    let clipboard = navigator.clipboard();

    match JsFuture::from(clipboard.read_text()).await {
        Ok(js_value) => js_value.as_string(),
        Err(_) => None,
    }
}

pub struct InviteCodeManager {
    page: Page,
    page_tx: Sender<Page>,
    page_rx: Receiver<Page>,
    error_rx: Receiver<String>,

    codes: Vec<Code>,
    filtered_codes: Vec<Code>,
    search_term: String,
    filter_status: FilterStatus,
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
    create_code_count_str: String,
    export_code_count_str: String,
    selected_codes: HashSet<String>,
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
            filter_status: FilterStatus::All,
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
            create_code_count_str: "1".to_string(),
            export_code_count_str: "1".to_string(),
            selected_codes: HashSet::new(),
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn default() -> Self {
        let (page_tx, page_rx) = crossbeam_channel::unbounded();
        let (error_tx, error_rx) = crossbeam_channel::unbounded();
        let (invite_code_tx, invite_code_rx) = crossbeam_channel::unbounded();
        let (qr_tx, qr_rx) = crossbeam_channel::unbounded();
        let client = Client::builder().build().unwrap();
        Self {
            page: Page::Login,
            page_tx,
            page_rx,
            error_rx,
            codes: vec![],
            filtered_codes: vec![],
            search_term: "".to_string(),
            filter_status: FilterStatus::All,
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
            create_code_count_str: "1".to_string(),
            export_code_count_str: "1".to_string(),
            selected_codes: HashSet::new(),
        }
    }
}

impl eframe::App for InviteCodeManager {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_zoom_factor(styles::get_dynamic_zoom_factor(ctx));
        egui::CentralPanel::default().show(ctx, |ui| {
            // Basic window styling.
            styles::set_text_color(ui);
            styles::render_title(ui, ctx, styles::FRAME_TITLE);

            let res = self.page_rx.try_recv();
            if let Ok(page) = res {
                self.page = page;
            }

            match &mut self.page {
                Page::Home => {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            self.show_home(ui);
                        });
                }
                Page::Login => {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            self.show_login(ui, ctx);
                        });
                }
                Page::QrVerify => {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            self.show_verify_qr(ui, ctx);
                        });
                }
                Page::QrValidate => {
                    egui::ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            self.show_validate_qr(ui, ctx);
                        });
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

#[derive(Deserialize, Serialize, Debug)]
pub struct InviteCodes {
    pub cursor: Option<String>,
    pub codes: Vec<Code>,
}

impl InviteCodeManager {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        egui_extras::install_image_loaders(&cc.egui_ctx);
        styles::setup_fonts(&cc.egui_ctx);
        styles::apply_global_style(&cc.egui_ctx);
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

    fn format_datetime(datetime_str: &str) -> String {
        if let Ok(dt) = DateTime::parse_from_rfc3339(datetime_str) {
            dt.with_timezone(&Utc).format("%b %d, %Y %H:%M").to_string()
        } else {
            datetime_str.to_string()
        }
    }

    fn invite_table_ui(&mut self, ui: &mut Ui) {
        if styles::is_mobile(ui.ctx()) {
            self.invite_list_mobile_ui(ui);
        } else {
            let table = TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::initial(30.0).at_least(30.0).resizable(false)) // Checkbox
                .column(Column::initial(100.0).at_least(80.0).resizable(true)) // Code
                .column(Column::initial(150.0).at_least(120.0).resizable(true)) // Created At
                .column(Column::auto().at_least(60.0).resizable(true)) // Used
                .column(Column::auto().at_least(80.0).resizable(true)) // Disabled
                .column(Column::remainder().at_least(100.0).resizable(true)) // Used By
                .column(Column::initial(150.0).at_least(120.0).resizable(true)) // Used At
                .column(Column::auto().at_least(100.0).resizable(true)) // Actions
                .min_scrolled_height(400.0)
                .vscroll(true);

            table
                .header(30.0, |mut header| {
                    header.col(|ui| {
                        let mut all_selected = !self.filtered_codes.is_empty()
                            && self
                                .filtered_codes
                                .iter()
                                .all(|c| self.selected_codes.contains(&c.code));
                        if ui.checkbox(&mut all_selected, "").clicked() {
                            if all_selected {
                                for code in &self.filtered_codes {
                                    self.selected_codes.insert(code.code.clone());
                                }
                            } else {
                                for code in &self.filtered_codes {
                                    self.selected_codes.remove(&code.code);
                                }
                            }
                        }
                    });
                    header.col(|ui| {
                        ui.vertical_centered(|ui| {
                            ui.strong("Code");
                        });
                    });
                    header.col(|ui| {
                        ui.vertical_centered(|ui| {
                            ui.strong("Created At");
                        });
                    });
                    header.col(|ui| {
                        ui.vertical_centered(|ui| {
                            ui.strong("Used");
                        });
                    });
                    header.col(|ui| {
                        ui.vertical_centered(|ui| {
                            ui.strong("Disabled");
                        });
                    });
                    header.col(|ui| {
                        ui.vertical_centered(|ui| {
                            ui.strong("Used By");
                        });
                    });
                    header.col(|ui| {
                        ui.vertical_centered(|ui| {
                            ui.strong("Used At");
                        });
                    });
                    header.col(|ui| {
                        ui.vertical_centered(|ui| {
                            ui.strong("Actions");
                        });
                    });
                })
                .body(|body| {
                    let codes = self.filtered_codes.clone();
                    let mut selected_codes = self.selected_codes.clone();
                    body.rows(40.0, codes.len(), |mut row| {
                        let index = row.index();
                        let code = &codes[index];

                        row.col(|ui| {
                            let mut is_selected = selected_codes.contains(&code.code);
                            if ui.checkbox(&mut is_selected, "").clicked() {
                                if is_selected {
                                    selected_codes.insert(code.code.clone());
                                } else {
                                    selected_codes.remove(&code.code);
                                }
                            }
                        });
                        row.col(|ui| {
                            ui.label(RichText::new(&code.code).monospace());
                        });
                        row.col(|ui| {
                            ui.label(Self::format_datetime(&code.created_at));
                        });
                        row.col(|ui| {
                            let used = code.available < 1 || !code.uses.is_empty();
                            let text = if used { "✔ Yes" } else { "❌ No" };
                            let color = if used {
                                egui::Color32::GREEN
                            } else {
                                ui.visuals().text_color()
                            };
                            ui.label(RichText::new(text).color(color));
                        });
                        row.col(|ui| {
                            let text = if code.disabled { "🚫 Yes" } else { "✅ No" };
                            let color = if code.disabled {
                                egui::Color32::RED
                            } else {
                                ui.visuals().text_color()
                            };
                            ui.label(RichText::new(text).color(color));
                        });
                        row.col(|ui| {
                            if let Some(usage) = code.uses.first() {
                                ui.label(&usage.used_by);
                            } else {
                                ui.label("-");
                            }
                        });
                        row.col(|ui| {
                            if let Some(usage) = code.uses.first() {
                                ui.label(Self::format_datetime(&usage.used_at));
                            } else {
                                ui.label("-");
                            }
                        });
                        row.col(|ui| {
                            ui.horizontal_centered(|ui| {
                                if !code.disabled {
                                    if ui.button("Disable").clicked() {
                                        self.disable_invite_code(code.code.clone());
                                    }
                                } else {
                                    ui.add_enabled(false, egui::Button::new("Disabled"));
                                }
                            });
                        });
                    });
                    self.selected_codes = selected_codes;
                });
        }
    }

    fn invite_list_mobile_ui(&mut self, ui: &mut Ui) {
        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                ui.vertical(|ui| {
                    let codes = self.filtered_codes.clone();
                    for code in codes {
                        styles::render_card(ui, |ui| {
                            ui.vertical(|ui| {
                                ui.horizontal(|ui| {
                                    let mut is_selected = self.selected_codes.contains(&code.code);
                                    if ui.checkbox(&mut is_selected, "").clicked() {
                                        if is_selected {
                                            self.selected_codes.insert(code.code.clone());
                                        } else {
                                            self.selected_codes.remove(&code.code);
                                        }
                                    }
                                    ui.label(RichText::new("Code:").strong());
                                    ui.label(RichText::new(&code.code).monospace());
                                });
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("Created:").strong());
                                    ui.label(Self::format_datetime(&code.created_at));
                                });

                                let used = code.available < 1 || !code.uses.is_empty();
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("Used:").strong());
                                    let text = if used { "✔ Yes" } else { "❌ No" };
                                    let color = if used {
                                        egui::Color32::GREEN
                                    } else {
                                        ui.visuals().text_color()
                                    };
                                    ui.label(RichText::new(text).color(color));
                                });

                                ui.horizontal(|ui| {
                                    ui.label(RichText::new("Disabled:").strong());
                                    let text = if code.disabled { "🚫 Yes" } else { "✅ No" };
                                    let color = if code.disabled {
                                        egui::Color32::RED
                                    } else {
                                        ui.visuals().text_color()
                                    };
                                    ui.label(RichText::new(text).color(color));
                                });

                                if let Some(usage) = code.uses.first() {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new("Used By:").strong());
                                        ui.label(&usage.used_by);
                                    });
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new("Used At:").strong());
                                        ui.label(Self::format_datetime(&usage.used_at));
                                    });
                                }

                                ui.add_space(4.0);
                                let ctx = ui.ctx().clone();
                                if !code.disabled {
                                    styles::render_button(ui, &ctx, "Disable", || {
                                        self.disable_invite_code(code.code.clone());
                                    });
                                } else {
                                    ui.add_enabled(false, egui::Button::new("Disabled"));
                                }
                            });
                        });
                        ui.add_space(8.0);
                    }
                });
            });
    }

    pub fn show_home(&mut self, ui: &mut Ui) {
        // 1. Handle logic/data updates first
        if let Ok(error_message) = self.error_rx.try_recv() {
            self.error_message = error_message;
        }

        if let Ok(invite_codes) = self.invite_code_rx.try_recv() {
            self.codes = invite_codes.codes;
            self.filter_invites();
        }

        let is_mobile = styles::is_mobile(ui.ctx());

        // 2. Render the top controls in a standard vertical layout
        ui.vertical(|ui| {
            if is_mobile {
                ui.set_max_width(ui.available_width());
            } else {
                ui.set_max_width(800.0); // Limit width on desktop for better readability
            }
            ui.vertical_centered(|ui| {
                if !self.error_message.is_empty() {
                    styles::render_error(ui, &self.error_message);
                }

                ui.add_space(8.0);

                let layout_func = |ui: &mut Ui| {
                    ui.spacing_mut().item_spacing.x = 10.0;
                    let ctx = ui.ctx().clone();
                    styles::render_button(ui, &ctx, "Refresh List", || {
                        self.get_invite_codes();
                    });

                    if is_mobile {
                        ui.add_space(8.0);
                    } else {
                        ui.separator();
                    }

                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Filter:").strong());
                        let filter_input =
                            styles::render_base_input(&mut self.search_term, false, true)
                                .hint_text("Search...");
                        let filter_input = if is_mobile {
                            filter_input.desired_width(ui.available_width() - 80.0)
                        } else {
                            filter_input
                        };
                        if ui.add(filter_input).changed() {
                            self.filter_invites();
                        }
                    });

                    if is_mobile {
                        ui.add_space(8.0);
                    }

                    egui::ComboBox::from_id_salt("status_filter")
                        .selected_text(self.filter_status.to_string())
                        .show_ui(ui, |ui| {
                            let mut changed = false;
                            changed |= ui
                                .selectable_value(&mut self.filter_status, FilterStatus::All, "All")
                                .clicked();
                            changed |= ui
                                .selectable_value(
                                    &mut self.filter_status,
                                    FilterStatus::Used,
                                    "Used",
                                )
                                .clicked();
                            changed |= ui
                                .selectable_value(
                                    &mut self.filter_status,
                                    FilterStatus::Unused,
                                    "Unused",
                                )
                                .clicked();
                            changed |= ui
                                .selectable_value(
                                    &mut self.filter_status,
                                    FilterStatus::Disabled,
                                    "Disabled",
                                )
                                .clicked();
                            if changed {
                                self.filter_invites();
                            }
                        });

                    if is_mobile {
                        ui.add_space(8.0);
                    } else {
                        ui.separator();
                    }

                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Count:").strong());
                        ui.add(
                            egui::TextEdit::singleline(&mut self.create_code_count_str)
                                .desired_width(if is_mobile { 60.0 } else { 40.0 }),
                        );
                    });

                    if is_mobile {
                        ui.add_space(8.0);
                    }

                    let ctx = ui.ctx().clone();
                    styles::render_button(ui, &ctx, "➕ Create Code", || {
                        self.create_invite_code();
                        self.get_invite_codes();
                    });

                    if is_mobile {
                        ui.add_space(8.0);
                    } else {
                        ui.separator();
                    }

                    let ctx = ui.ctx().clone();
                    styles::render_button(ui, &ctx, "📥 Export all", || {
                        let codes: Vec<String> =
                            self.codes.iter().map(|c| c.code.clone()).collect();
                        self.download_txt(codes, &ctx);
                    });

                    if is_mobile {
                        ui.add_space(8.0);
                    } else {
                        ui.separator();
                    }

                    ui.horizontal(|ui| {
                        ui.label(RichText::new("Export N:").strong());
                        ui.add(
                            egui::TextEdit::singleline(&mut self.export_code_count_str)
                                .desired_width(if is_mobile { 60.0 } else { 40.0 }),
                        );
                    });

                    let ctx = ui.ctx().clone();
                    styles::render_button(ui, &ctx, "📥 Export .txt", || {
                        let count = self.export_code_count_str.parse::<usize>().unwrap_or(0);
                        let codes: Vec<String> = self
                            .codes
                            .iter()
                            .take(count)
                            .map(|c| c.code.clone())
                            .collect();
                        self.download_txt(codes, &ctx);
                    });

                    if !self.selected_codes.is_empty() {
                        if is_mobile {
                            ui.add_space(8.0);
                        } else {
                            ui.separator();
                        }
                        let ctx = ui.ctx().clone();
                        let count = self.selected_codes.len();
                        styles::render_button(
                            ui,
                            &ctx,
                            &format!("📥 Export selection ({})", count),
                            || {
                                self.export_selected_codes(&ctx);
                            },
                        );
                    }
                };

                let available_width = ui.available_width();
                let controls_width = if is_mobile {
                    available_width
                } else {
                    1000.0f32.min(available_width * 0.95)
                };

                ui.allocate_ui_with_layout(
                    egui::vec2(controls_width, ui.available_height()),
                    egui::Layout::top_down(egui::Align::Center),
                    |ui| {
                        if is_mobile {
                            ui.vertical(layout_func);
                        } else {
                            ui.horizontal(layout_func);
                        }
                    },
                );

                ui.add_space(8.0);
            });
        });

        ui.separator();

        // 3. The CRITICAL part: Use the remaining height for the table.
        // We use a separate Ui for the table that occupies all remaining space.
        ui.vertical_centered(|ui| {
            let available_width = ui.available_width();
            let table_width = if is_mobile {
                available_width
            } else {
                1000.0f32.min(available_width * 0.95)
            };

            ui.allocate_ui_with_layout(
                egui::vec2(table_width, ui.available_height()),
                egui::Layout::top_down(egui::Align::Center),
                |ui| {
                    self.invite_table_ui(ui);
                },
            );
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
                let _ = error_tx.send(format!("Received HTTP Status: {status}"));
                return;
            }
            let invite_codes = res.json::<InviteCodes>().await;
            match invite_codes {
                Ok(invite_codes) => {
                    let _ = invite_code_tx.send(invite_codes);
                }
                Err(e) => {
                    let _ = error_tx.send(format!("Error: {e}"));
                }
            }
        });
    }

    fn filter_invites(&mut self) {
        self.filtered_codes.clear();
        self.selected_codes.clear();
        let search_term = self.search_term.to_lowercase();
        for code in self.codes.clone() {
            // Search filter
            let matches_search = if search_term.is_empty() {
                true
            } else {
                let code_match = code.code.to_lowercase().contains(&search_term);
                let used_by_match = code
                    .uses
                    .iter()
                    .any(|u| u.used_by.to_lowercase().contains(&search_term));
                code_match || used_by_match
            };

            // Status filter
            let used = code.available < 1 || !code.uses.is_empty();
            let matches_status = match self.filter_status {
                FilterStatus::All => true,
                FilterStatus::Used => used,
                FilterStatus::Unused => !used && !code.disabled,
                FilterStatus::Disabled => code.disabled,
            };

            if matches_search && matches_status {
                self.filtered_codes.push(code.clone());
            }
        }
    }

    fn export_selected_codes(&mut self, ctx: &egui::Context) {
        let codes: Vec<String> = self.selected_codes.iter().cloned().collect();
        self.download_txt(codes, ctx);
        self.selected_codes.clear();
    }

    fn download_txt(&self, codes: Vec<String>, ctx: &egui::Context) {
        let export_text = codes.join("\n");
        ctx.copy_text(export_text.clone());

        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                if let Some(document) = window.document() {
                    let parts = js_sys::Array::new();
                    parts.push(&wasm_bindgen::JsValue::from_str(&export_text));

                    let mut blob_props = web_sys::BlobPropertyBag::new();
                    blob_props.set_type("text/plain");

                    if let Ok(blob) =
                        web_sys::Blob::new_with_str_sequence_and_options(&parts, &blob_props)
                    {
                        if let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob) {
                            if let Ok(a) = document.create_element("a") {
                                if let Ok(a) =
                                    wasm_bindgen::JsCast::dyn_into::<web_sys::HtmlAnchorElement>(a)
                                {
                                    a.set_href(&url);
                                    a.set_download("invite_codes.txt");
                                    a.click();
                                    let _ = web_sys::Url::revoke_object_url(&url);
                                }
                            }
                        }
                    }
                }
            }
        }

        #[cfg(not(target_arch = "wasm32"))]
        {
            let handle = rfd::FileDialog::new()
                .set_file_name("invite_codes.txt")
                .add_filter("Text", &["txt"])
                .save_file();

            if let Some(path) = handle {
                let _ = std::fs::write(path, export_text);
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
                let _ = error_tx.send(format!("Received HTTP Status: {status}"));
            }
        });
    }

    fn create_invite_code(&mut self) {
        let count = self.create_code_count_str.parse::<i32>().unwrap_or(1);
        let endpoint = self.invite_backend.clone() + CREATE_INVITE_CODES;
        let client = self.client.clone();
        let error_tx = self.error_tx.clone();
        let body = CreateInviteCodeBody {
            code_count: count,
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
                let _ = error_tx.send(format!("Received HTTP Status: {status}"));
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

            styles::render_card(ui, |ui| {
                ui.set_max_width(400.0);
                ui.vertical_centered(|ui| {
                    styles::render_input(
                        ui,
                        "Enter 2FA Code",
                        &mut self.otp_code,
                        false,
                        Some("000000"),
                    );

                    // Display current error message, if exists
                    if !self.error_message.is_empty() {
                        styles::render_error(ui, &self.error_message);
                    }

                    ui.add_space(10.0);
                    styles::render_button(ui, ctx, "Verify & Login", || {
                        self.validate_otp();
                    });
                });
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

    #[tracing::instrument(skip(self, ui, ctx), fields(page = "QrVerify"))]
    pub fn show_verify_qr(&mut self, ui: &mut Ui, ctx: &Context) {
        tracing::info!("show_verify_qr");
        if !self.generated_otp {
            tracing::info!("Generating OTP");
            self.generated_otp = true;
            self.generate_otp();
        }

        let res = self.qr_rx.try_recv();
        if res.is_ok() {
            tracing::info!("Got QR code");
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
            tracing::info!("Got error message");
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

    #[tracing::instrument(skip(self), fields(page = "QrVerify"))]
    fn verify_otp(&mut self) {
        tracing::info!("verify_otp");
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
                let _ = error_tx.send(format!("Received HTTP Status: {status}"));
                return;
            }
            let _ = page_tx.send(Page::Home);
        });
    }

    pub fn show_login(&mut self, ui: &mut Ui, ctx: &Context) {
        styles::render_subtitle(ui, ctx, "Login!");

        // Check for new error messages
        if let Ok(error_message) = self.error_rx.try_recv() {
            self.error_message = error_message;
        }

        ui.vertical_centered(|ui| {
            styles::render_card(ui, |ui| {
                if styles::is_mobile(ui.ctx()) {
                    ui.set_max_width(ui.available_width());
                } else {
                    ui.set_max_width(400.0);
                }
                ui.vertical_centered(|ui| {
                    styles::render_input(
                        ui,
                        "Invite Manager Endpoint",
                        &mut self.invite_backend,
                        false,
                        None,
                    );

                    let input_layout = |ui: &mut Ui| {
                        styles::render_input(ui, "Username", &mut self.username, false, None);
                        let ctx = ui.ctx().clone();
                        styles::render_button(ui, &ctx.clone(), "📋 Paste", || {
                            #[cfg(target_arch = "wasm32")]
                            {
                                // For web/mobile web
                                wasm_bindgen_futures::spawn_local(async move {
                                    if let Some(text) = read_clipboard_web().await {
                                        ctx.data_mut(|d| {
                                            d.insert_temp(
                                                egui::Id::new("username_paste_data"),
                                                text,
                                            )
                                        });
                                        ctx.request_repaint();
                                    }
                                });
                            }

                            #[cfg(not(target_arch = "wasm32"))]
                            {
                                if let Some(text) = ctx.input(|i| {
                                    i.events.iter().find_map(|e| {
                                        if let egui::Event::Paste(s) = e {
                                            Some(s.clone())
                                        } else {
                                            None
                                        }
                                    })
                                }) {
                                    println!("{:?}", text);
                                    self.username = text;
                                    ctx.request_repaint();
                                }
                            }
                        });
                    };

                    if styles::is_mobile(ui.ctx()) {
                        ui.vertical(input_layout);
                    } else {
                        ui.horizontal(input_layout);
                    }

                    let password_layout = |ui: &mut Ui| {
                        styles::render_input(ui, "Password", &mut self.password, true, None);
                        let ctx = ui.ctx().clone();
                        styles::render_button(ui, &ctx.clone(), "📋 Paste", || {
                            #[cfg(target_arch = "wasm32")]
                            {
                                // For web/mobile web
                                wasm_bindgen_futures::spawn_local(async move {
                                    if let Some(text) = read_clipboard_web().await {
                                        ctx.data_mut(|d| {
                                            d.insert_temp(
                                                egui::Id::new("password_paste_data"),
                                                text,
                                            )
                                        });
                                        ctx.request_repaint();
                                    }
                                });
                            }

                            #[cfg(not(target_arch = "wasm32"))]
                            {
                                // For egui 0.28+, use:
                                if let Some(text) = ctx.input(|i| {
                                    i.events.iter().find_map(|e| {
                                        if let egui::Event::Paste(s) = e {
                                            Some(s.clone())
                                        } else {
                                            None
                                        }
                                    })
                                }) {
                                    println!("{:?}", text);
                                    self.password = text;
                                    ctx.request_repaint();
                                }
                            }
                        });
                    };

                    if styles::is_mobile(ui.ctx()) {
                        ui.vertical(password_layout);
                    } else {
                        ui.horizontal(password_layout);
                    }

                    #[cfg(target_arch = "wasm32")]
                    if let Some(text) = ui
                        .ctx()
                        .data(|d| d.get_temp::<String>(egui::Id::new("username_paste_data")))
                    {
                        self.username = text.clone();
                        ui.ctx()
                            .data_mut(|d| d.remove::<String>(egui::Id::new("username_paste_data")));
                        ctx.request_repaint();
                    }

                    #[cfg(target_arch = "wasm32")]
                    if let Some(text) = ui
                        .ctx()
                        .data(|d| d.get_temp::<String>(egui::Id::new("password_paste_data")))
                    {
                        self.password = text.clone();
                        ui.ctx()
                            .data_mut(|d| d.remove::<String>(egui::Id::new("password_paste_data")));
                        ctx.request_repaint();
                    }

                    // Display current error message, if exists
                    if !self.error_message.is_empty() {
                        styles::render_error(ui, &self.error_message);
                    }

                    ui.add_space(10.0);
                    styles::render_button(ui, ctx, "Submit", || {
                        self.login();
                    });
                });
            });
        });
    }

    #[tracing::instrument(skip(self), fields(page = "Login"))]
    fn login(&mut self) {
        tracing::info!("login");
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
            tracing::info!("Sending login request");
            let res = match client
                .post(login_endpoint)
                .header("Content-Type", "application/json")
                .json(&LoginRequest { username, password })
                .send()
                .await
            {
                Ok(response) => response,
                Err(err) => {
                    let _ = error_tx.send(format!("Error: {err}"));
                    return;
                }
            };

            if !res.status().is_success() {
                let status = res.status();
                let _ = error_tx.send(format!("Received HTTP Status: {status}"));
                return;
            }

            if res.status() == StatusCode::OK {
                let _ = page_tx.send(Page::QrValidate);
            } else if res.status() == StatusCode::from_u16(201).unwrap() {
                let _ = page_tx.send(Page::QrVerify);
            }
        });
    }
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
