use egui::{Color32, CornerRadius, Frame, Margin, Response, Sense, Stroke, pos2, vec2};
use crate::theme::ZedTheme;

/// Animated toggle switch
pub fn toggle(ui: &mut egui::Ui, ctx: &egui::Context, id_str: &str, value: &mut bool) -> bool {
    let desired = vec2(40.0, 20.0);
    let (rect, resp) = ui.allocate_exact_size(desired, Sense::click());

    let anim_id = ui.id().with(id_str);
    let anim = ctx.animate_bool_with_time(anim_id, *value, 0.18);

    let painter = ui.painter_at(rect);

    let r = (ZedTheme::PINK.r() as f32 * anim + ZedTheme::BG_HOVER.r() as f32 * (1.0 - anim)) as u8;
    let g = (ZedTheme::PINK.g() as f32 * anim + ZedTheme::BG_HOVER.g() as f32 * (1.0 - anim)) as u8;
    let b = (ZedTheme::PINK.b() as f32 * anim + ZedTheme::BG_HOVER.b() as f32 * (1.0 - anim)) as u8;
    let track_color = Color32::from_rgb(r, g, b);

    painter.rect_filled(rect, CornerRadius::same(10), track_color);
    painter.rect_stroke(
        rect,
        CornerRadius::same(10),
        Stroke::new(1.0, ZedTheme::BORDER),
        egui::epaint::StrokeKind::Outside,
    );

    let thumb_x = rect.left() + 3.0 + anim * (rect.width() - 20.0 - 3.0);
    let thumb_center = pos2(thumb_x + 7.0, rect.center().y);
    painter.circle_filled(thumb_center, 7.0, Color32::WHITE);

    if *value {
        painter.circle_filled(
            thumb_center,
            9.0,
            Color32::from_rgba_premultiplied(233, 30, 140, 30),
        );
    }

    let changed = resp.clicked();
    if changed {
        *value = !*value;
    }
    changed
}

/// Section card with title and border — forces full available width
pub fn section_card<R>(
    ui: &mut egui::Ui,
    title: &str,
    add_contents: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    let avail_w = ui.available_width();
    Frame::new()
        .fill(ZedTheme::BG_CARD)
        .corner_radius(CornerRadius::same(10))
        .stroke(Stroke::new(1.0, ZedTheme::BORDER))
        .inner_margin(Margin::same(14))
        .show(ui, |ui| {
            ui.set_min_width(avail_w - 2.0); // -2 for border stroke on each side
            ui.horizontal(|ui| {
                let (dot_rect, _) = ui.allocate_exact_size(vec2(8.0, 8.0), Sense::hover());
                ui.painter().circle_filled(dot_rect.center(), 3.5, ZedTheme::PINK);
                ui.add_space(4.0);
                ui.label(egui::RichText::new(title)
                    .color(ZedTheme::TEXT_BRIGHT)
                    .size(12.5)
                    .strong());
            });
            ui.add_space(8.0);
            ui.separator();
            ui.add_space(8.0);
            add_contents(ui)
        }).inner
}

/// Toggle row with label on left and toggle on right
pub fn toggle_row(
    ui: &mut egui::Ui,
    ctx: &egui::Context,
    id: &str,
    label: &str,
    hint: Option<&str>,
    value: &mut bool,
) {
    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(egui::RichText::new(label).color(ZedTheme::TEXT_NORMAL).size(12.5));
            if let Some(h) = hint {
                ui.label(egui::RichText::new(h).color(ZedTheme::TEXT_DEAD).size(10.0));
            }
        });
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            toggle(ui, ctx, id, value);
        });
    });
    ui.add_space(2.0);
}

/// Styled text input — uses Margin::symmetric to fix egui 0.34 API
pub fn styled_input(ui: &mut egui::Ui, label: &str, hint: &str, value: &mut String) -> Response {
    ui.vertical(|ui| {
        ui.label(egui::RichText::new(label).color(ZedTheme::TEXT_DIM).size(11.0));
        ui.add_space(3.0);
        ui.add(
            egui::TextEdit::singleline(value)
                .hint_text(egui::RichText::new(hint).color(ZedTheme::TEXT_DEAD))
                .desired_width(f32::INFINITY)
                .margin(Margin::symmetric(10, 6))
                .background_color(ZedTheme::BG_INPUT),
        )
    }).inner
}

/// Pink primary button with hover glow
pub fn pink_button(ui: &mut egui::Ui, ctx: &egui::Context, id: &str, label: &str) -> Response {
    let desired = vec2(ui.available_width().min(240.0).max(160.0), 38.0);
    let (rect, resp) = ui.allocate_exact_size(desired, Sense::click());

    let is_hovered = resp.hovered();
    let is_pressed = resp.is_pointer_button_down_on();

    let anim_id = ui.id().with(id).with("hover");
    let anim = ctx.animate_bool_with_time(anim_id, is_hovered, 0.12);

    let painter = ui.painter_at(rect);

    if anim > 0.01 {
        painter.rect_filled(
            rect.expand(anim * 4.0),
            CornerRadius::same(10),
            Color32::from_rgba_premultiplied(233, 30, 140, (anim * 30.0) as u8),
        );
    }

    let base = if is_pressed {
        ZedTheme::PINK_DIM
    } else {
        Color32::from_rgb(
            (ZedTheme::PINK_DIM.r() as f32 + anim * (ZedTheme::PINK.r() as f32 - ZedTheme::PINK_DIM.r() as f32)) as u8,
            (ZedTheme::PINK_DIM.g() as f32 + anim * (ZedTheme::PINK.g() as f32 - ZedTheme::PINK_DIM.g() as f32)) as u8,
            (ZedTheme::PINK_DIM.b() as f32 + anim * (ZedTheme::PINK.b() as f32 - ZedTheme::PINK_DIM.b() as f32)) as u8,
        )
    };
    painter.rect_filled(rect, CornerRadius::same(8), base);
    painter.rect_stroke(
        rect,
        CornerRadius::same(8),
        Stroke::new(1.0, ZedTheme::PINK),
        egui::epaint::StrokeKind::Outside,
    );

    painter.text(
        rect.center(),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::new(13.0, egui::FontFamily::Proportional),
        Color32::WHITE,
    );

    resp
}
