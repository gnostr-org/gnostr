use std::time::Duration;

use crate::cube::{
    commands::{init_terminal, /*quit_terminal,*/ reset_terminal},
    handlers::{
        config::CompleteConfig,
        event::{Config, /*Event,*/ Events, Key},
    },
    //ui::draw_ui,
};

pub async fn ui_driver(config: CompleteConfig) {
    let original_hook = std::panic::take_hook();

    std::panic::set_hook(Box::new(move |panic| {
        reset_terminal();
        original_hook(panic);
    }));

    let events = Events::with_config(Config {
        exit_key: Key::Null,
        tick_rate: Duration::from_millis(config.terminal.tick_delay),
    })
    .await;

    let mut terminal = init_terminal(&config.frontend);

    terminal.clear().unwrap();

    //loop {
    //    terminal.draw(|frame| draw_ui(frame, &config)).unwrap();

    //    if matches!(events.next().await, Some(Event::Input(Key::Char('q')))) {
    //        quit_terminal(terminal);

    //        break;
    //    }
    //}

    reset_terminal();
}
