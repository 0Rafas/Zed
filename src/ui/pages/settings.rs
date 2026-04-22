use egui::{Color32, CornerRadius, Frame, Margin};
use crate::state::AppState;
use crate::theme::ZedTheme;
use crate::ui::widgets::{section_card, toggle_row};

pub fn draw_settings(ui: &mut egui::Ui, ctx: &egui::Context, state: &mut AppState) {
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            ui.add_space(20.0);

            ui.horizontal(|ui| {
                ui.add_space(12.0);
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("Settings").color(ZedTheme::TEXT_BRIGHT).size(20.0).strong());
                    ui.label(egui::RichText::new("Application preferences").color(ZedTheme::TEXT_DIM).size(11.5));
                });
            });

            ui.add_space(20.0);

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.add_space(12.0);
                let w = (ui.available_width() - 12.0).max(1.0);

                ui.vertical(|ui| {
                    ui.set_width(w);

                    // ── Preferences ───────────────────────────────────────────
                    section_card(ui, "Preferences", |ui| {
                        toggle_row(ui, ctx, "st_sav",  "Auto-save Config",  Some("Save settings on exit"),          &mut state.settings.auto_save);
                        toggle_row(ui, ctx, "st_not",  "Notifications",     Some("Show status notifications"),       &mut state.settings.notifications);

                        // Manual save button (always available regardless of auto-save)
                        ui.add_space(8.0);
                        ui.horizontal(|ui| {
                            if ui.button(
                                egui::RichText::new("[>] Save Now")
                                    .color(ZedTheme::PINK)
                                    .size(11.5)
                            ).clicked() {
                                state.save_config();
                            }
                            ui.add_space(12.0);
                            ui.label(
                                egui::RichText::new("Config saved to %APPDATA%\\ZedBuilder\\config.json")
                                    .color(ZedTheme::TEXT_DEAD)
                                    .size(10.0),
                            );
                        });
                    });

                    ui.add_space(10.0);

                    // ── Appearance ────────────────────────────────────────────
                    section_card(ui, "Appearance", |ui| {
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new("Pink Intensity")
                                    .color(ZedTheme::TEXT_NORMAL)
                                    .size(12.5),
                            );
                            ui.add_space(8.0);

                            // Slider
                            let slider = egui::Slider::new(&mut state.settings.theme_pink, 0.3..=1.0)
                                .show_value(false)
                                .trailing_fill(true);
                            let resp = ui.add(slider);

                            ui.add_space(8.0);

                            // Live preview swatch
                            let alpha = (state.settings.theme_pink * 255.0) as u8;
                            let (swatch_rect, _) = ui.allocate_exact_size(
                                egui::vec2(18.0, 18.0),
                                egui::Sense::hover(),
                            );
                            ui.painter().rect_filled(
                                swatch_rect,
                                egui::CornerRadius::same(4),
                                Color32::from_rgba_premultiplied(233, 30, 140, alpha),
                            );

                            let _ = resp;
                        });

                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new("Controls the glow intensity of the pink accent color")
                                .color(ZedTheme::TEXT_DEAD)
                                .size(10.5),
                        );
                    });

                    ui.add_space(10.0);

                    // ── About ─────────────────────────────────────────────────
                    section_card(ui, "About ZED Stealer", |ui| {
                        info_row(ui, "Version",  "0.1.0");
                        info_row(ui, "Engine",   "Rust + C++17");
                        info_row(ui, "UI",       "egui 0.34 / eframe");
                        info_row(ui, "Crypto",   "AES-256-GCM + DPAPI");
                        info_row(ui, "Platform", "Windows x64");

                        ui.add_space(10.0);

                        Frame::new()
                            .fill(Color32::from_rgba_premultiplied(233, 30, 140, 10))
                            .corner_radius(CornerRadius::same(8))
                            .inner_margin(Margin::same(12))
                            .show(ui, |ui| {
                                ui.set_min_width(ui.available_width());
                                ui.label(
                                    egui::RichText::new(
                                        "ZED Stealer v0.1 — Professional Payload Builder\n\
                                         Built for authorized red-team and security testing only.",
                                    )
                                    .color(ZedTheme::TEXT_DIM)
                                    .size(11.0),
                                );
                            });
                    });

                    ui.add_space(10.0);

                    // ── Features Overview ─────────────────────────────────────
                    section_card(ui, "Features Overview", |ui| {
                        feature_row(ui, "Discord",   "Token extraction + Nitro validation");
                        feature_row(ui, "Telegram",  "tdata session file copy");
                        feature_row(ui, "Browsers",  "DPAPI + AES-GCM decrypt (Chromium) + NSS (Firefox)");
                        feature_row(ui, "System",    "OS, CPU, GPU, RAM, Disk, WiFi, Clipboard");
                        feature_row(ui, "Network",   "Public IP, Country, ISP, City (ip-api.com)");
                        feature_row(ui, "Delivery",  "Discord Webhooks + Telegram Bot API");
                        feature_row(ui, "Evasion",   "Anti-Debug, Anti-VM, AMSI/ETW patch, Syscalls");
                    });
                });
            });

            ui.add_space(20.0);
        });
}

fn info_row(ui: &mut egui::Ui, key: &str, val: &str) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(format!("{key}:")).color(ZedTheme::TEXT_DIM).size(12.0));
        ui.add_space(6.0);
        ui.label(egui::RichText::new(val).color(ZedTheme::TEXT_NORMAL).size(12.0));
    });
    ui.add_space(3.0);
}

fn feature_row(ui: &mut egui::Ui, module: &str, desc: &str) {
    ui.horizontal(|ui| {
        ui.label(egui::RichText::new(format!("[+] {module}")).color(ZedTheme::PINK).size(11.5).monospace());
        ui.add_space(8.0);
        ui.label(egui::RichText::new(desc).color(ZedTheme::TEXT_DIM).size(11.0));
    });
    ui.add_space(2.0);
}
