use gpui::{Pixels, Size, px, size};

const PREFERRED_WIDTH: f32 = 1280.0;
const PREFERRED_HEIGHT: f32 = 800.0;
const SHARE_OF_SCREEN: f32 = 0.9;

pub fn default_size(visible: Size<Pixels>) -> Size<Pixels> {
    size(
        px(PREFERRED_WIDTH).min(visible.width * SHARE_OF_SCREEN),
        px(PREFERRED_HEIGHT).min(visible.height * SHARE_OF_SCREEN),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_big_screen_gets_the_preferred_size() {
        assert_eq!(default_size(size(px(2560.0), px(1400.0))), size(px(1280.0), px(800.0)));
    }

    #[test]
    fn a_small_screen_gets_ninety_percent_of_it() {
        assert_eq!(default_size(size(px(1000.0), px(600.0))), size(px(900.0), px(540.0)));
    }

    #[test]
    fn width_and_height_are_limited_on_their_own() {
        assert_eq!(default_size(size(px(2560.0), px(600.0))), size(px(1280.0), px(540.0)));
        assert_eq!(default_size(size(px(1000.0), px(1400.0))), size(px(900.0), px(800.0)));
    }

    #[test]
    fn the_size_is_always_smaller_than_the_screen() {
        for (width, height) in [(640.0, 480.0), (1366.0, 728.0), (1920.0, 1032.0), (3840.0, 2100.0)] {
            let visible = size(px(width), px(height));
            let fitted = default_size(visible);
            assert!(fitted.width < visible.width && fitted.height < visible.height);
        }
    }
}
