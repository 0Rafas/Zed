#[allow(deprecated)]
use egui::{Color32, CornerRadius, Rect, Stroke, pos2, vec2};
use crate::state::{AppState, Page};
use crate::theme::ZedTheme;
use crate::ui::{
    titlebar::{draw_titlebar, TitlebarAction},
    sidebar::draw_sidebar,
    pages::{home, builder, compiler, settings},
};

pub struct ZedApp {
    pub state: AppState,
}

impl Default for ZedApp {
    fn default() -> Self {
        Self {
            // Load persisted config on startup; fall back to defaults
            state: AppState::load_config(),
        }
    }
}

impl eframe::App for ZedApp {
    fn clear_color(&self, _visuals: &egui::Visuals) -> [f32; 4] {
        egui::Rgba::TRANSPARENT.to_array()
    }

    fn on_exit(&mut self) {
        if self.state.settings.auto_save {
            self.state.save_config();
        }
    }

    #[allow(deprecated)]
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        // Only force immediate repaint while the build thread is running.
        // Animations (sidebar hover, toggles, grid pulse) call request_repaint
        // themselves via ctx.animate_*. This avoids pegging the CPU at idle.
        if self.state.is_building {
            ctx.request_repaint();
        } else {
            // Repaint ~30fps to keep the grid pulse alive without wasting CPU
            ctx.request_repaint_after(std::time::Duration::from_millis(33));
        }

        let full_rect = ui.max_rect();

        // Paint window chrome directly — no Frame::show() wrapper
        {
            let p = ui.painter();
            p.add(egui::epaint::Shadow {
                offset: [0, 16].into(),
                blur: 48,
                spread: 0,
                color: Color32::from_black_alpha(180),
            }.as_shape(full_rect, CornerRadius::same(10)));
            p.rect_filled(full_rect, CornerRadius::same(10), ZedTheme::BG_VOID);
            p.rect_stroke(
                full_rect,
                CornerRadius::same(10),
                Stroke::new(1.0, ZedTheme::BORDER),
                egui::epaint::StrokeKind::Inside,
            );
        }

        ui.set_clip_rect(full_rect);

        const TB_H: f32 = 38.0;
        const SB_W: f32 = crate::ui::sidebar::SIDEBAR_W;

        // Explicitly defined rects for each region — prevents height-collapse bugs
        let titlebar_rect = Rect::from_min_size(
            full_rect.min,
            vec2(full_rect.width(), TB_H),
        );
        let sidebar_rect = Rect::from_min_size(
            pos2(full_rect.left(), full_rect.top() + TB_H),
            vec2(SB_W, full_rect.height() - TB_H),
        );
        let content_rect = Rect::from_min_max(
            pos2(full_rect.left() + SB_W, full_rect.top() + TB_H),
            full_rect.max,
        );

        // Titlebar
        ui.allocate_ui_at_rect(titlebar_rect, |ui| {
            match draw_titlebar(&ctx, ui) {
                TitlebarAction::Close    => ctx.send_viewport_cmd(egui::ViewportCommand::Close),
                TitlebarAction::Minimize => ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true)),
                TitlebarAction::None     => {}
            }
        });

        // Sidebar
        ui.allocate_ui_at_rect(sidebar_rect, |ui| {
            draw_sidebar(&ctx, ui, &mut self.state);
        });

        // Content
        let theme_pink = self.state.settings.theme_pink;
        ui.allocate_ui_at_rect(content_rect, |ui| {
            draw_bg_grid(ui, theme_pink);
            match self.state.current_page {
                Page::Home     => home::draw_home(ui, &ctx, &self.state),
                Page::Builder  => builder::draw_builder(ui, &ctx, &mut self.state),
                Page::Compiler => compiler::draw_compiler(ui, &ctx, &mut self.state),
                Page::Settings => settings::draw_settings(ui, &ctx, &mut self.state),
            }
        });
    }
}

fn draw_bg_grid(ui: &mut egui::Ui, theme_pink: f32) {
    let t = ui.input(|i| i.time) as f32;
    let rect = ui.max_rect();
    let painter = ui.painter_at(rect);

    let grid_alpha = (4.0 * theme_pink) as u8;
    let grid_color = Color32::from_rgba_premultiplied(233, 30, 140, grid_alpha);
    let spacing = 32.0f32;

    let mut x = rect.left();
    while x < rect.right() {
        painter.line_segment(
            [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
            Stroke::new(0.5, grid_color),
        );
        x += spacing;
    }

    let mut y = rect.top();
    while y < rect.bottom() {
        painter.line_segment(
            [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
            Stroke::new(0.5, grid_color),
        );
        y += spacing;
    }

    // Animated corner glow — scaled by theme_pink
    let pulse = (t * 1.5).sin() * 0.5 + 0.5;
    let glow_origin = rect.right_bottom() + egui::vec2(-10.0, -10.0);
    for i in 0..3 {
        let r = 60.0 + (i as f32) * 40.0 + pulse * 20.0;
        let a = (15.0 - (i as f32) * 4.0) * pulse * theme_pink;
        painter.circle_stroke(
            glow_origin,
            r,
            Stroke::new(0.5, Color32::from_rgba_premultiplied(233, 30, 140, a as u8)),
        );
    }
}
