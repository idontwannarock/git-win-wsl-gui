#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod bench;
mod branches;
mod detail;
mod graph;
mod home;
mod keyboard;
mod motion;
mod theme;
mod working;
#[cfg(test)]
mod patch_parse_tests;

use gitcore::trace;
use gpui::{App, AppContext, WindowOptions};
use gpui_component::Root;
use home::Home;

fn start_trace() {
    let Some(base) = std::env::var_os("LOCALAPPDATA") else {
        return;
    };
    let path = std::path::PathBuf::from(base)
        .join("git-win-wsl-gui")
        .join("app.log");
    if trace::init(&path).is_err() {
        return;
    }
    trace::record(|| format!("start version {}", env!("CARGO_PKG_VERSION")));
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        trace::record(|| {
            format!("panic: {info} | {}", std::backtrace::Backtrace::force_capture())
        });
        previous(info);
    }));
}

fn main() {
    start_trace();
    let initial_path = std::env::args().nth(1);
    gpui_platform::application().run(move |cx: &mut App| {
        gpui_component::init(cx);
        keyboard::bind(cx);
        motion::follow_system(cx);
        theme::load_fonts(cx);
        theme::apply(cx);
        cx.open_window(WindowOptions::default(), |window, cx| {
            let home = cx.new(|cx| Home::new(window, cx));
            if let Some(path) = initial_path {
                home.update(cx, |home, cx| home.open(path, cx));
            }
            cx.new(|cx| Root::new(home, window, cx))
        })
        .expect("open window");
        cx.activate(true);
    });
}
