use tao::event_loop::{ControlFlow, EventLoop};
use tray_icon::{menu::MenuEvent, TrayIconBuilder};

fn main() {
    let context = gnostr_tray_icon::default_tray_context();
    let icon = gnostr_tray_icon::load_gnostr_icon_tinted(gnostr_tray_icon::tray_icon_tint_from_env());
    let event_loop = EventLoop::new();
    let tray_menu = context.menu;
    let gnostr_item = context.gnostr_item;
    let quit_item = context.quit_item;

    let _tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip(gnostr_tray_icon::DEFAULT_TOOLTIP)
        .with_icon(icon)
        .build()
        .unwrap();

    let menu_channel = MenuEvent::receiver();

    event_loop.run(move |_event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        if let Ok(event) = menu_channel.try_recv() {
            if event.id == quit_item.id() {
                *control_flow = ControlFlow::Exit;
            } else if event.id == gnostr_item.id() {
                if let Err(error) = gnostr_tray_icon::spawn_terminal_command(
                    gnostr_tray_icon::gnostr_command_line(),
                ) {
                    eprintln!("failed to launch gnostr: {error}");
                }
            }
        }
    });
}
