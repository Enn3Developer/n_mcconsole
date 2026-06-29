use crossterm::event::{self, Event as CtEvent, KeyEvent, KeyEventKind};
use n_mcconsole_core::message::{Envelope, Message, Tagged};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, RecvError, SendError, Sender};
use std::thread;
use std::time::Duration;

/// Represents an event that can be fired using an [EventWriter]
pub enum Event {
    /// An input represented by its key event
    Input(KeyEvent),
    /// A message sent to the event bus
    Bus(Envelope),
    Tick,
    Resize,
}

/// Reads incoming events
pub struct EventReader {
    pub(crate) rx: Receiver<Event>,
}

impl EventReader {
    pub fn read(&self) -> Result<Event, RecvError> {
        self.rx.recv()
    }
}

/// Emits events
#[derive(Clone)]
pub struct EventWriter {
    pub(crate) tx: Sender<Event>,
}

impl EventWriter {
    /// Emit an input event
    pub fn input(&self, key: KeyEvent) -> Result<(), SendError<Event>> {
        self.tx.send(Event::Input(key))
    }

    /// Send a broadcast message
    pub fn bus<T: Message>(&self, msg: T) -> Result<(), SendError<Event>> {
        self.tx.send(Event::Bus(Envelope::new(msg)))
    }

    /// Send a message to deliver to a specific recipient
    pub fn bus_tagged<P: Send + 'static>(
        &self,
        tag: u64,
        payload: P,
    ) -> Result<(), SendError<Event>> {
        self.bus(Tagged::new(tag, payload))
    }

    /// Send a tick
    pub fn tick(&self) -> Result<(), SendError<Event>> {
        self.tx.send(Event::Tick)
    }

    /// Send a resize event
    pub fn resize(&self) -> Result<(), SendError<Event>> {
        self.tx.send(Event::Resize)
    }
}

/// Create a new couple of writer and reader for an event channel
///
/// You can clone the writer and send to other threads
pub fn create_event_channel() -> (EventWriter, EventReader) {
    let (tx, rx) = mpsc::channel::<Event>();

    (EventWriter { tx }, EventReader { rx })
}

/// Spawn a new thread which reads events from the system and emits input and resize events
pub fn spawn_input(writer: EventWriter) {
    thread::spawn(move || {
        loop {
            match event::read() {
                Ok(CtEvent::Key(k)) if k.kind == KeyEventKind::Press => {
                    if writer.input(k).is_err() {
                        break;
                    }
                }
                Ok(CtEvent::Resize(_, _)) => {
                    if writer.resize().is_err() {
                        break;
                    }
                }
                Ok(_) => {}
                Err(_) => break,
            }
        }
    });
}

/// The backbone of the tick system, spawns a new thread that fires the tick event multiple times per second
pub fn spawn_ticker(writer: EventWriter, period: Duration) {
    thread::spawn(move || {
        loop {
            thread::sleep(period);
            if writer.tick().is_err() {
                break;
            }
        }
    });
}
