#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod bench;
mod branches;
mod detail;
mod graph;
mod keyboard;
mod motion;
mod repo_view;
mod shell;
mod tab_list;
mod tab_session;
mod tab_title;
mod theme;
mod window_size;
mod working;
#[cfg(test)]
mod patch_parse_tests;

use gitcore::trace;
use gpui::{App, AppContext, Bounds, WindowBounds, WindowOptions};
use gpui_component::Root;
use shell::Shell;

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
        let window_bounds = cx.primary_display().map(|display| {
            let visible = display.visible_bounds();
            let fitted = window_size::default_size(visible.size);
            WindowBounds::Windowed(Bounds::centered_at(visible.center(), fitted))
        });
        let options = WindowOptions { window_bounds, ..WindowOptions::default() };
        cx.open_window(options, |window, cx| {
            let shell = cx.new(|cx| Shell::new(window, cx));
            shell.update(cx, |shell, cx| shell.restore(window, cx));
            if let Some(path) = initial_path {
                shell.update(cx, |shell, cx| shell.open(path, window, cx));
            }
            cx.new(|cx| Root::new(shell, window, cx))
        })
        .expect("open window");
        cx.activate(true);
    });
}
