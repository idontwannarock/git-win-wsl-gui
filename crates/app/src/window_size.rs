const PREFERRED: (f32, f32) = (1280.0, 800.0);
const SHARE_OF_SCREEN: f32 = 0.9;

pub fn default_size(visible: (f32, f32)) -> (f32, f32) {
    (
        PREFERRED.0.min(visible.0 * SHARE_OF_SCREEN),
        PREFERRED.1.min(visible.1 * SHARE_OF_SCREEN),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_big_screen_gets_the_preferred_size() {
        assert_eq!(default_size((2560.0, 1400.0)), (1280.0, 800.0));
    }

    #[test]
    fn a_1080p_screen_leaves_room_for_the_title_bar_and_the_taskbar() {
        let (width, height) = default_size((1920.0, 1032.0));
        assert!(width <= 1920.0 * 0.9);
        assert!(height <= 1032.0 * 0.9);
    }

    #[test]
    fn a_small_screen_gets_ninety_percent_of_it() {
        assert_eq!(default_size((1000.0, 600.0)), (900.0, 540.0));
    }

    #[test]
    fn the_size_never_exceeds_the_screen() {
        for visible in [(640.0, 480.0), (1366.0, 728.0), (1920.0, 1032.0), (3840.0, 2100.0)] {
            let (width, height) = default_size(visible);
            assert!(width < visible.0 && height < visible.1, "{visible:?} -> {width}x{height}");
        }
    }
}
