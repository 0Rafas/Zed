use egui::{Color32, CornerRadius, Rect, Sense, Stroke, pos2, vec2};
use crate::theme::ZedTheme;

pub fn draw_titlebar(ctx: &egui::Context, ui: &mut egui::Ui) -> TitlebarAction {
    let mut action = TitlebarAction::None;

    let bar_height = 38.0;
    let (rect, response) = ui.allocate_exact_size(
        vec2(ui.available_width(), bar_height),
        Sense::click_and_drag(),
    );

    if response.drag_started() {
        ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag);
    }

    let painter = ui.painter_at(rect);

    // Background
    painter.rect_filled(rect, CornerRadius::ZERO, ZedTheme::BG_SIDEBAR);

    // Bottom border
    painter.line_segment(
        [pos2(rect.left(), rect.bottom()), pos2(rect.right(), rect.bottom())],
        Stroke::new(1.0, ZedTheme::BORDER),
    );

    // Pink top accent line
    painter.line_segment(
        [pos2(rect.left(), rect.top()), pos2(rect.left() + 180.0, rect.top())],
        Stroke::new(1.5, ZedTheme::PINK),
    );

    // Draw ZED logo
    draw_zed_logo(&painter, pos2(rect.left() + 14.0, rect.top() + 10.0));

    // "ZED STEALER" text
    painter.text(
        pos2(rect.left() + 38.0, rect.center().y),
        egui::Align2::LEFT_CENTER,
        "ZED STEALER",
        egui::FontId::new(13.0, egui::FontFamily::Monospace),
        ZedTheme::TEXT_BRIGHT,
    );

    // Version
    painter.text(
        pos2(rect.left() + 148.0, rect.center().y + 1.0),
        egui::Align2::LEFT_CENTER,
        "v0.1",
        egui::FontId::new(10.0, egui::FontFamily::Proportional),
        ZedTheme::TEXT_DEAD,
    );

    // Window control buttons
    let btn_size = vec2(38.0, bar_height);
    let close_rect = Rect::from_min_size(
        pos2(rect.right() - btn_size.x, rect.top()),
        btn_size,
    );
    let min_rect = Rect::from_min_size(
        pos2(rect.right() - btn_size.x * 2.0, rect.top()),
        btn_size,
    );

    // Minimize button
    let min_resp = ui.interact(min_rect, ui.id().with("min_btn"), Sense::click());
    let min_bg = if min_resp.hovered() { ZedTheme::BG_HOVER } else { Color32::TRANSPARENT };
    painter.rect_filled(min_rect, CornerRadius::ZERO, min_bg);
    let mx = min_rect.center();
    painter.line_segment(
        [pos2(mx.x - 5.0, mx.y), pos2(mx.x + 5.0, mx.y)],
        Stroke::new(1.5, if min_resp.hovered() { ZedTheme::TEXT_BRIGHT } else { ZedTheme::TEXT_DIM }),
    );
    if min_resp.clicked() {
        action = TitlebarAction::Minimize;
    }

    // Close button
    let close_resp = ui.interact(close_rect, ui.id().with("close_btn"), Sense::click());
    let close_bg = if close_resp.hovered() {
        Color32::from_rgb(180, 20, 20)
    } else {
        Color32::TRANSPARENT
    };
    painter.rect_filled(close_rect, CornerRadius::ZERO, close_bg);
    let cx = close_rect.center();
    let x_off = 5.0_f32;
    painter.line_segment(
        [pos2(cx.x - x_off, cx.y - x_off), pos2(cx.x + x_off, cx.y + x_off)],
        Stroke::new(1.5, if close_resp.hovered() { Color32::WHITE } else { ZedTheme::TEXT_DIM }),
    );
    painter.line_segment(
        [pos2(cx.x + x_off, cx.y - x_off), pos2(cx.x - x_off, cx.y + x_off)],
        Stroke::new(1.5, if close_resp.hovered() { Color32::WHITE } else { ZedTheme::TEXT_DIM }),
    );
    if close_resp.clicked() {
        action = TitlebarAction::Close;
    }

    action
}

fn draw_zed_logo(painter: &egui::Painter, origin: egui::Pos2) {
    let p = origin;
    let s = 16.0_f32;
    let pink = ZedTheme::PINK;
    let stroke = Stroke::new(2.0, pink);

    // Z shape
    painter.line_segment([pos2(p.x, p.y), pos2(p.x + s, p.y)], stroke);
    painter.line_segment([pos2(p.x + s, p.y), pos2(p.x, p.y + s)], stroke);
    painter.line_segment([pos2(p.x, p.y + s), pos2(p.x + s, p.y + s)], stroke);

    // Glow
    let glow = Stroke::new(4.0, Color32::from_rgba_premultiplied(233, 30, 140, 25));
    painter.line_segment([pos2(p.x, p.y), pos2(p.x + s, p.y)], glow);
    painter.line_segment([pos2(p.x + s, p.y), pos2(p.x, p.y + s)], glow);
    painter.line_segment([pos2(p.x, p.y + s), pos2(p.x + s, p.y + s)], glow);
}

pub enum TitlebarAction {
    None,
    Minimize,
    Close,
}
