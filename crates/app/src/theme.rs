use gitcore::refs::RefKind;
use gpui::{App, Hsla, Rgba, px};
use gpui_component::{Theme, ThemeMode};
use std::sync::LazyLock;

#[derive(Clone, Copy)]
pub struct Oklch {
    lightness: f32,
    chroma: f32,
    hue: f32,
}

impl Oklch {
    pub const fn new(lightness_percent: f32, chroma: f32, hue: f32) -> Self {
        Self {
            lightness: lightness_percent / 100.0,
            chroma,
            hue,
        }
    }

    fn linear(self) -> [f32; 3] {
        let (sin, cos) = self.hue.to_radians().sin_cos();
        let (a, b) = (self.chroma * cos, self.chroma * sin);
        let l = (self.lightness + 0.396_337_78 * a + 0.215_803_76 * b).powi(3);
        let m = (self.lightness - 0.105_561_346 * a - 0.063_854_17 * b).powi(3);
        let s = (self.lightness - 0.089_484_18 * a - 1.291_485_5 * b).powi(3);
        [
            4.076_741_7 * l - 3.307_711_6 * m + 0.230_969_93 * s,
            -1.268_438 * l + 2.609_757_4 * m - 0.341_319_4 * s,
            -0.004_196_086 * l - 0.703_418_6 * m + 1.707_614_7 * s,
        ]
        .map(|channel| channel.clamp(0.0, 1.0))
    }

    pub fn rgba(self) -> Rgba {
        let [r, g, b] = self.linear().map(encode);
        Rgba { r, g, b, a: 1.0 }
    }

    pub fn hsla(self) -> Hsla {
        self.rgba().into()
    }
}

fn encode(linear: f32) -> f32 {
    if linear <= 0.003_130_8 {
        12.92 * linear
    } else {
        1.055 * linear.powf(1.0 / 2.4) - 0.055
    }
}

pub const PAPER: Oklch = Oklch::new(17.0, 0.008, 250.0);
pub const PAPER_2: Oklch = Oklch::new(21.0, 0.009, 250.0);
pub const INK: Oklch = Oklch::new(95.0, 0.005, 250.0);
pub const MUTED: Oklch = Oklch::new(71.0, 0.01, 250.0);
pub const RULE: Oklch = Oklch::new(31.0, 0.01, 250.0);
pub const CONTROL: Oklch = Oklch::new(55.0, 0.01, 250.0);
pub const ACCENT: Oklch = Oklch::new(82.0, 0.15, 80.0);
pub const OK: Oklch = Oklch::new(76.0, 0.15, 150.0);
pub const BAD: Oklch = Oklch::new(70.0, 0.17, 25.0);

pub const LANES: [Oklch; 6] = [
    Oklch::new(76.0, 0.12, 240.0),
    Oklch::new(76.0, 0.12, 320.0),
    ACCENT,
    OK,
    BAD,
    Oklch::new(76.0, 0.1, 195.0),
];

pub const RADIUS: f32 = 10.0;

static LANE_COLORS: LazyLock<[Rgba; 6]> = LazyLock::new(|| LANES.map(Oklch::rgba));

pub fn lane_color(lane: usize) -> Rgba {
    LANE_COLORS[lane % LANES.len()]
}

pub fn chip_colors(kind: RefKind, is_head: bool) -> (Hsla, Hsla) {
    let (background, text) = match (kind, is_head) {
        (RefKind::Branch, true) => (ACCENT, PAPER),
        (RefKind::Branch, false) => (RULE, INK),
        (RefKind::RemoteBranch, _) => (PAPER_2, MUTED),
        (RefKind::Tag, _) => (PAPER_2, ACCENT),
    };
    (background.hsla(), text.hsla())
}

pub fn apply(cx: &mut App) {
    Theme::change(ThemeMode::Dark, None, cx);
    Theme::update(cx, |theme| {
        let colors = &mut theme.colors;
        colors.background = PAPER.hsla();
        colors.foreground = INK.hsla();
        colors.muted = PAPER_2.hsla();
        colors.muted_foreground = MUTED.hsla();
        colors.border = RULE.hsla();
        colors.input = CONTROL.hsla();
        colors.ring = ACCENT.hsla();
        colors.caret = INK.hsla();
        colors.selection = ACCENT.hsla().opacity(0.3);
        colors.accent = PAPER_2.hsla();
        colors.accent_foreground = INK.hsla();
        colors.primary = ACCENT.hsla();
        colors.primary_hover = ACCENT.hsla().opacity(0.9);
        colors.primary_active = ACCENT.hsla().opacity(0.8);
        colors.primary_foreground = PAPER.hsla();
        colors.button_primary = ACCENT.hsla();
        colors.button_primary_hover = ACCENT.hsla().opacity(0.9);
        colors.button_primary_active = ACCENT.hsla().opacity(0.8);
        colors.button_primary_foreground = PAPER.hsla();
        for (base, hover, active, foreground) in [
            (&mut colors.secondary, &mut colors.secondary_hover, &mut colors.secondary_active, &mut colors.secondary_foreground),
            (&mut colors.button, &mut colors.button_hover, &mut colors.button_active, &mut colors.button_foreground),
        ] {
            *base = PAPER_2.hsla();
            *hover = RULE.hsla();
            *active = RULE.hsla();
            *foreground = INK.hsla();
        }
        colors.danger = BAD.hsla();
        colors.danger_foreground = PAPER.hsla();
        colors.success = OK.hsla();
        colors.success_foreground = PAPER.hsla();
        colors.warning = ACCENT.hsla();
        colors.warning_foreground = PAPER.hsla();
        colors.info = LANES[0].hsla();
        colors.info_foreground = PAPER.hsla();
        colors.list = PAPER.hsla();
        colors.list_even = PAPER.hsla();
        colors.list_head = PAPER_2.hsla();
        colors.list_hover = PAPER_2.hsla();
        colors.list_active = RULE.hsla();
        colors.popover = PAPER_2.hsla();
        colors.popover_foreground = INK.hsla();
        colors.title_bar = PAPER_2.hsla();
        colors.title_bar_border = RULE.hsla();
        colors.window_border = RULE.hsla();
        theme.radius_lg = px(RADIUS);
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEXT_PAIRS: [(Oklch, Oklch); 12] = [
        (INK, PAPER),
        (INK, PAPER_2),
        (INK, RULE),
        (MUTED, PAPER),
        (MUTED, PAPER_2),
        (ACCENT, PAPER),
        (ACCENT, PAPER_2),
        (OK, PAPER),
        (OK, PAPER_2),
        (BAD, PAPER),
        (BAD, PAPER_2),
        (PAPER, ACCENT),
    ];

    fn luminance(color: Oklch) -> f32 {
        let [r, g, b] = color.linear();
        0.2126 * r + 0.7152 * g + 0.0722 * b
    }

    fn contrast(a: Oklch, b: Oklch) -> f32 {
        let (light, dark) = (luminance(a).max(luminance(b)), luminance(a).min(luminance(b)));
        (light + 0.05) / (dark + 0.05)
    }

    const WHITE: Oklch = Oklch::new(100.0, 0.0, 0.0);
    const BLACK: Oklch = Oklch::new(0.0, 0.0, 0.0);

    fn close(actual: f32, expected: f32) -> bool {
        (actual - expected).abs() < 0.01
    }

    #[test]
    fn white_and_black_convert_to_the_ends_of_the_scale() {
        let white = WHITE.rgba();
        let black = BLACK.rgba();
        assert!(close(white.r, 1.0) && close(white.g, 1.0) && close(white.b, 1.0));
        assert!(close(black.r, 0.0) && close(black.g, 0.0) && close(black.b, 0.0));
    }

    #[test]
    fn a_mid_gray_converts_to_the_known_srgb_value() {
        let gray = Oklch::new(50.0, 0.0, 0.0).rgba();
        assert!(close(gray.r, 0.388) && close(gray.g, 0.388) && close(gray.b, 0.388));
    }

    #[test]
    fn a_saturated_color_converts_to_the_known_srgb_value() {
        let red = Oklch::new(62.8, 0.2577, 29.23).rgba();
        assert!(close(red.r, 1.0) && close(red.g, 0.0) && close(red.b, 0.0));
    }

    #[test]
    fn black_on_white_has_the_maximum_contrast() {
        assert!(close(contrast(WHITE, BLACK), 21.0));
        assert!(close(contrast(BLACK, WHITE), 21.0));
    }

    #[test]
    fn every_text_pair_meets_the_4_5_floor() {
        for (index, (text, background)) in TEXT_PAIRS.into_iter().enumerate() {
            let ratio = contrast(text, background);
            assert!(ratio >= 4.5, "text pair {index}: {ratio}");
        }
    }

    #[test]
    fn every_control_border_focus_ring_and_graph_line_meets_the_3_floor() {
        let mut graphics = vec![("focus ring", ACCENT), ("control border", CONTROL)];
        graphics.extend(LANES.map(|lane| ("lane", lane)));
        for (name, color) in graphics {
            for background in [PAPER, PAPER_2] {
                let ratio = contrast(color, background);
                assert!(ratio >= 3.0, "{name}: {ratio}");
            }
        }
    }
}
