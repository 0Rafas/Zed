use egui::{Color32, FontId, FontFamily, Stroke, Vec2, Visuals};

pub struct ZedTheme;

impl ZedTheme {
    pub const BG_VOID: Color32       = Color32::from_rgb(5, 5, 8);
    pub const BG_PANEL: Color32      = Color32::from_rgb(10, 10, 16);
    pub const BG_CARD: Color32       = Color32::from_rgb(15, 15, 22);
    pub const BG_INPUT: Color32      = Color32::from_rgb(12, 12, 18);
    pub const BG_HOVER: Color32      = Color32::from_rgb(22, 22, 32);
    pub const BG_SIDEBAR: Color32    = Color32::from_rgb(8, 8, 13);

    pub const PINK: Color32          = Color32::from_rgb(233, 30, 140);
    pub const PINK_DIM: Color32      = Color32::from_rgb(140, 18, 84);
    pub const PINK_GLOW: Color32     = Color32::from_rgba_premultiplied(233, 30, 140, 40);
    pub const PINK_SUBTLE: Color32   = Color32::from_rgba_premultiplied(233, 30, 140, 15);

    pub const TEXT_BRIGHT: Color32   = Color32::from_rgb(240, 240, 248);
    pub const TEXT_NORMAL: Color32   = Color32::from_rgb(185, 185, 200);
    pub const TEXT_DIM: Color32      = Color32::from_rgb(100, 100, 120);
    pub const TEXT_DEAD: Color32     = Color32::from_rgb(55, 55, 70);

    pub const BORDER: Color32        = Color32::from_rgb(25, 25, 38);
    pub const BORDER_ACTIVE: Color32 = Color32::from_rgba_premultiplied(233, 30, 140, 80);

    pub const SUCCESS: Color32       = Color32::from_rgb(40, 200, 120);
    pub const WARNING: Color32       = Color32::from_rgb(255, 165, 40);
    pub const DANGER: Color32        = Color32::from_rgb(220, 50, 50);

    pub fn apply(ctx: &egui::Context) {
        let mut style = (*ctx.global_style()).clone();

        style.spacing.item_spacing      = Vec2::new(8.0, 6.0);
        style.spacing.button_padding    = Vec2::new(14.0, 7.0);
        style.spacing.window_margin     = egui::Margin::same(0);
        style.spacing.indent            = 16.0;
        style.spacing.scroll            = egui::style::ScrollStyle::solid();

        style.visuals = Self::visuals();

        style.text_styles = {
            let mut t = std::collections::BTreeMap::new();
            t.insert(egui::TextStyle::Small,   FontId::new(11.0, FontFamily::Proportional));
            t.insert(egui::TextStyle::Body,    FontId::new(13.0, FontFamily::Proportional));
            t.insert(egui::TextStyle::Button,  FontId::new(13.0, FontFamily::Proportional));
            t.insert(egui::TextStyle::Heading, FontId::new(18.0, FontFamily::Proportional));
            t.insert(egui::TextStyle::Monospace, FontId::new(12.0, FontFamily::Monospace));
            t
        };

        ctx.set_global_style(style);
    }

    fn visuals() -> Visuals {
        let mut v = Visuals::dark();

        v.window_fill        = Self::BG_PANEL;
        v.panel_fill         = Self::BG_PANEL;
        v.faint_bg_color     = Self::BG_CARD;
        v.extreme_bg_color   = Self::BG_INPUT;
        v.code_bg_color      = Self::BG_INPUT;

        v.window_stroke      = Stroke::new(1.0, Self::BORDER);
        v.window_shadow      = egui::epaint::Shadow {
            offset: [0, 8].into(),
            blur: 32,
            spread: 0,
            color: Color32::from_black_alpha(160),
        };

        v.widgets.noninteractive.bg_fill    = Self::BG_PANEL;
        v.widgets.noninteractive.fg_stroke  = Stroke::new(1.0, Self::TEXT_DIM);
        v.widgets.noninteractive.bg_stroke  = Stroke::new(1.0, Self::BORDER);

        v.widgets.inactive.bg_fill          = Self::BG_CARD;
        v.widgets.inactive.fg_stroke        = Stroke::new(1.0, Self::TEXT_NORMAL);
        v.widgets.inactive.bg_stroke        = Stroke::new(1.0, Self::BORDER);

        v.widgets.hovered.bg_fill           = Self::BG_HOVER;
        v.widgets.hovered.fg_stroke         = Stroke::new(1.0, Self::TEXT_BRIGHT);
        v.widgets.hovered.bg_stroke         = Stroke::new(1.0, Self::PINK_DIM);

        v.widgets.active.bg_fill            = Color32::from_rgb(20, 12, 18);
        v.widgets.active.fg_stroke          = Stroke::new(1.0, Self::PINK);
        v.widgets.active.bg_stroke          = Stroke::new(1.5, Self::PINK);

        v.selection.bg_fill                 = Self::PINK_DIM;
        v.selection.stroke                  = Stroke::new(1.0, Self::PINK);

        v.hyperlink_color                   = Self::PINK;
        v.override_text_color               = Some(Self::TEXT_NORMAL);

        v
    }
}
