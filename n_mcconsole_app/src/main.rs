mod terminal;

use crate::terminal::{Tui, init_terminal, restore_terminal};
use n_mcconsole_core::config::Config;
use n_mcconsole_core::executor::Executor;
use n_mcconsole_core::message::{Envelope, Tick};
use n_mcconsole_event_bus::app::App;
use n_mcconsole_event_bus::event::{
    Event, EventReader, create_event_channel, spawn_input, spawn_ticker,
};
use n_mcconsole_event_bus::job::JobControl;
use n_mcconsole_executor::local::LocalExecutor;
use n_mcconsole_executor::remote::SshExecutor;
use n_mcconsole_executor::{Target, parse_target};
use n_mcconsole_scenes::tabbed_scene::{TabBuilder, TabbedScene};
use n_mcconsole_scenes::test_scene::TestScene;
use std::io;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use std::time::Duration;

fn main() -> io::Result<()> {
    let executor: Arc<dyn Executor> = match parse_target() {
        Target::Local => Arc::new(LocalExecutor),
        Target::Remote { host, opts } => Arc::new(SshExecutor { host, opts }),
    };

    let (writer, reader) = create_event_channel();
    spawn_input(writer.clone());
    spawn_ticker(writer.clone(), Duration::from_millis(100));

    let jobs = JobControl {
        writer: writer.clone(),
        executor,
        next: Arc::new(AtomicU64::new(1)),
    };

    let mut app = App::new(Config {}, jobs);
    let tabs = TabBuilder::new(&mut app).add_tab(TestScene, "Test").build();
    let tabbed_scene = TabbedScene::new(tabs);
    app.push_scene(tabbed_scene);

    let prev = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = restore_terminal();
        prev(info);
    }));

    let mut terminal = init_terminal()?;
    let result = run(&mut app, &reader, &mut terminal);
    restore_terminal()?;
    result
}

fn run(app: &mut App, rx: &EventReader, terminal: &mut Tui) -> io::Result<()> {
    while app.running {
        terminal.draw(|f| app.render(f))?;

        match rx.read() {
            Ok(Event::Input(key)) => app.handle_input(key),
            Ok(Event::Bus(env)) => {
                app.enqueue(env);
                app.dispatch_all();
            }
            Ok(Event::Tick) => {
                app.enqueue(Envelope::new(Tick));
                app.dispatch_all();
            }
            Ok(Event::Resize) => {}
            Err(_) => break,
        }
    }
    Ok(())
}
