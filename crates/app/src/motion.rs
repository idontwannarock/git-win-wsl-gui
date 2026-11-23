use gpui::App;
use windows_sys::Win32::UI::WindowsAndMessaging::{SPI_GETCLIENTAREAANIMATION, SystemParametersInfoW};

fn system_animations_enabled() -> bool {
    let mut enabled: i32 = 1;
    let read = unsafe {
        SystemParametersInfoW(SPI_GETCLIENTAREAANIMATION, 0, (&raw mut enabled).cast(), 0)
    };
    read == 0 || enabled != 0
}

pub fn follow_system(cx: &mut App) {
    cx.set_reduce_motion(!system_animations_enabled());
}

