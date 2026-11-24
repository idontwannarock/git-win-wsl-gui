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
mod tab_title;
mod theme;
mod working;
#[cfg(test)]
mod patch_parse_tests;

use gitcore::trace;
use gpui::{App, AppContext, WindowOptions};
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
        cx.open_window(WindowOptions::default(), |window, cx| {
            let shell = cx.new(|cx| Shell::new(window, cx));
            if let Some(path) = initial_path {
                shell.update(cx, |shell, cx| shell.open(path, window, cx));
            }
            cx.new(|cx| Root::new(shell, window, cx))
        })
        .expect("open window");
        cx.activate(true);
    });
}
