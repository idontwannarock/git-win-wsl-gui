mod bench;
mod graph;
mod home;

use gpui::{App, AppContext, WindowOptions};
use gpui_component::Root;
use home::Home;

fn main() {
    let initial_path = std::env::args().nth(1);
    gpui_platform::application().run(move |cx: &mut App| {
        gpui_component::init(cx);
        cx.open_window(WindowOptions::default(), |window, cx| {
            let home = cx.new(|_| Home::new());
            if let Some(path) = initial_path {
                home.update(cx, |home, cx| home.open(path, cx));
            }
            cx.new(|cx| Root::new(home, window, cx))
        })
        .expect("open window");
        cx.activate(true);
    });
}
