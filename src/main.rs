use clap::Parser;
/// Listen for keyboard, mouse and trackpad events and send them to the model.
use device_query::{self, DeviceEventsHandler};
use device_query::{DeviceEvents, Keycode, MouseButton};
use std::sync::mpsc::{Sender, channel};
use std::time::Duration;

// TODO: Add test. Verifies it does not hang, verifies it actually receives inputs

#[tokio::main]
async fn main() {
    let args = Args::parse();
    internal_main(args).await;
}

#[derive(Parser, Debug)]
struct Args {
    poll_intervals: u64,
}

async fn internal_main(args: Args) {
    let (tx, _) = channel::<(InputEvent, Timestamp)>();
    let poll_intervals = Duration::from_millis(args.poll_intervals);
    send_inputs(tx, poll_intervals).await;
}

/// User event, where the user moves the mouse, presses the mmouse buttons
#[derive(Debug, Default, Clone, Copy)]
pub enum InputEvent {
    KB(Keycode),
    MouseButton(MouseButton),
    #[default]
    MouseMovement,
}

// TODO: Add diff calculations
struct Timestamp(jiff::Timestamp);

impl Timestamp {
    pub fn now() -> Self {
        let now = jiff::Timestamp::now();
        Self(now)
    }
}

async fn send_inputs(channel: Sender<(InputEvent, Timestamp)>, duration: Duration) {
    let handler = DeviceEventsHandler::new(duration).unwrap();
    let cloned = channel.clone();
    setup_mouse_move_handler(&handler, cloned);

    let cloned = channel.clone();
    setup_mouse_down_handler(&handler, cloned);

    setup_key_down_handler(handler, channel);

    loop {
        tokio::time::sleep(duration / 2).await;
    }
}

fn setup_key_down_handler(handler: DeviceEventsHandler, tx: Sender<(InputEvent, Timestamp)>) {
    let _kd_guard = handler.on_key_down(move |kd| {
        tx.send((InputEvent::KB(*kd), Timestamp::now())).unwrap();
    });
}

fn setup_mouse_down_handler(
    handler: &DeviceEventsHandler,
    cloned: Sender<(InputEvent, Timestamp)>,
) {
    let _md_guard = handler.on_mouse_down(move |md| {
        cloned
            .send((InputEvent::MouseButton(*md), Timestamp::now()))
            .unwrap();
    });
}

fn setup_mouse_move_handler(handler: &DeviceEventsHandler, tx: Sender<(InputEvent, Timestamp)>) {
    let _mm_guard = handler.on_mouse_move(move |_| {
        tx.send((InputEvent::MouseMovement, Timestamp::now()))
            .unwrap();
    });
}

    loop {
        tokio::time::sleep(duration / 2).await;
    }
}
