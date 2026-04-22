use egui::{Color32, CornerRadius, FontFamily, FontId, Rect, Sense, Stroke, pos2, vec2};
use crate::state::{AppState, Page};
use crate::theme::ZedTheme;

pub const SIDEBAR_W: f32 = 72.0;

pub fn draw_sidebar(ctx: &egui::Context, ui: &mut egui::Ui, state: &mut AppState) {
    let available = ui.available_size();
    let sidebar_rect = Rect::from_min_size(
        ui.cursor().min,
        vec2(SIDEBAR_W, available.y),
    );

    ui.allocate_exact_size(vec2(SIDEBAR_W, available.y), Sense::hover());

    let painter = ui.painter_at(sidebar_rect);

    // Background
    painter.rect_filled(sidebar_rect, CornerRadius::ZERO, ZedTheme::BG_SIDEBAR);

    // Right border
    painter.line_segment(
        [
            pos2(sidebar_rect.right(), sidebar_rect.top()),
            pos2(sidebar_rect.right(), sidebar_rect.bottom()),
        ],
        Stroke::new(1.0, ZedTheme::BORDER),
    );

    // Top pink accent
    painter.line_segment(
        [pos2(sidebar_rect.left(), sidebar_rect.top()), pos2(sidebar_rect.right(), sidebar_rect.top())],
        Stroke::new(2.0, Color32::from_rgba_premultiplied(233, 30, 140, 60)),
    );

    let pages = [Page::Home, Page::Builder, Page::Compiler, Page::Settings];
    let btn_h  = 58.0;
    let start_y = sidebar_rect.top() + 16.0;

    for (i, page) in pages.iter().enumerate() {
        let y = start_y + (i as f32) * (btn_h + 6.0);
        let btn_rect = Rect::from_min_size(
            pos2(sidebar_rect.left() + 6.0, y),
            vec2(SIDEBAR_W - 12.0, btn_h),
        );

        let is_active  = state.current_page == *page;
        let resp = ui.interact(btn_rect, ui.id().with(format!("sb_{:?}", page)), Sense::click());
        let is_hovered = resp.hovered();

        let anim_id = ui.id().with(format!("sb_a_{:?}", page));
        let target  = if is_active { 1.0f32 } else if is_hovered { 0.5 } else { 0.0 };
        let anim    = ctx.animate_value_with_time(anim_id, target, 0.15);

        // Background fill
        let bg = if is_active {
            Color32::from_rgba_premultiplied(233, 30, 140, 22)
        } else {
            Color32::from_rgba_premultiplied(255, 255, 255, (anim * 10.0) as u8)
        };
        painter.rect_filled(btn_rect, CornerRadius::same(8), bg);

        // Active indicator bar on left
        if is_active {
            painter.rect_filled(
                Rect::from_min_size(pos2(sidebar_rect.left() + 2.0, y + 12.0), vec2(3.0, btn_h - 24.0)),
                CornerRadius::same(2),
                ZedTheme::PINK,
            );
        }

        // Hover border glow
        if anim > 0.01 {
            painter.rect_stroke(
                btn_rect,
                CornerRadius::same(8),
                Stroke::new(1.0, Color32::from_rgba_premultiplied(233, 30, 140, (anim * 80.0) as u8)),
                egui::epaint::StrokeKind::Outside,
            );
        }

        let icon_color = if is_active {
            ZedTheme::PINK
        } else {
            Color32::from_rgb(
                (80.0 + anim * 120.0) as u8,
                (80.0 + anim * 80.0) as u8,
                (100.0 + anim * 80.0) as u8,
            )
        };

        let label_color = if is_active {
            ZedTheme::TEXT_BRIGHT
        } else {
            Color32::from_rgb(
                (65.0 + anim * 120.0) as u8,
                (65.0 + anim * 120.0) as u8,
                (80.0 + anim * 120.0) as u8,
            )
        };

        // Icon (short ASCII text — works with default font)
        painter.text(
            pos2(btn_rect.center().x, btn_rect.top() + 20.0),
            egui::Align2::CENTER_CENTER,
            page.icon_text(),
            FontId::new(15.0, FontFamily::Monospace),
            icon_color,
        );

        // Label
        painter.text(
            pos2(btn_rect.center().x, btn_rect.bottom() - 13.0),
            egui::Align2::CENTER_CENTER,
            page.label(),
            FontId::new(9.0, FontFamily::Proportional),
            label_color,
        );

        if resp.clicked() {
            state.current_page = *page;
        }
    }

    // Bottom watermark
    painter.text(
        pos2(sidebar_rect.center().x, sidebar_rect.bottom() - 10.0),
        egui::Align2::CENTER_CENTER,
        "ZED",
        FontId::new(9.0, FontFamily::Monospace),
        ZedTheme::TEXT_DEAD,
    );
}
