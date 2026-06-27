use crossterm::event::{self, Event as CtEvent, KeyEvent, KeyEventKind};
use n_mcconsole_core::message::Envelope;
use std::sync::mpsc::Sender;
use std::thread;
use std::time::Duration;

pub enum Event {
    Input(KeyEvent),
    Bus(Envelope),
    Tick,
    Resize,
}

pub fn spawn_input(tx: Sender<Event>) {
    thread::spawn(move || {
        loop {
            match event::read() {
                Ok(CtEvent::Key(k)) if k.kind == KeyEventKind::Press => {
                    if tx.send(Event::Input(k)).is_err() {
                        break;
                    }
                }
                Ok(CtEvent::Resize(_, _)) => {
                    if tx.send(Event::Resize).is_err() {
                        break;
                    }
                }
                Ok(_) => {}
                Err(_) => break,
            }
        }
    });
}

pub fn spawn_ticker(tx: Sender<Event>, period: Duration) {
    thread::spawn(move || {
        loop {
            thread::sleep(period);
            if tx.send(Event::Tick).is_err() {
                break;
            }
        }
    });
}
