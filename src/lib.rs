use clap::Parser;
/// Listen for keyboard and mouse events and send them down a channel.
use derive_more::{self, Display};
use device_query::{self, CallbackGuard, DeviceEventsHandler};
use device_query::{DeviceEvents, Keycode, MouseButton};
use std::mem::drop;
use std::sync::LazyLock;
use std::sync::mpsc::{Sender, channel};
use std::time::Duration;
use tokio::task::JoinHandle as TokioJoinHandle;
use tokio_util::sync::CancellationToken;

#[derive(Parser, Debug)]
pub struct Args {}

impl Args {}

/// We must use a *very* small duration for polling. If not, we might lose events! (and it makes
/// testing with simulated inputs unreliable)
pub const INPUT_POLL_INTERVAL: Duration = Duration::from_millis(10);
/// Unfortunately, the device has access to a global (static) event_loop. To enable testing, we must
/// initialisze it once.
pub static INPUT_HANDLER: LazyLock<DeviceEventsHandler> =
    LazyLock::new(|| DeviceEventsHandler::new(INPUT_POLL_INTERVAL).unwrap());

pub async fn internal_main(_args: Args) {
    let (tx, rx) = channel::<(PotentialInputEvent, Timestamp)>();
    let cancel = CancellationToken::new();
    let handler = &INPUT_HANDLER;
    let handle = launch_send_inputs_task(handler, tx, cancel.clone()).await;
    dbg!("out of internal main");
    let _ = tokio::join!(handle);
}

pub async fn launch_send_inputs_task(
    handler: &'static DeviceEventsHandler,
    tx: Sender<(PotentialInputEvent, Timestamp)>,
    cancel: CancellationToken,
) -> TokioJoinHandle<()> {
    let ret = tokio::spawn(async move {
        send_inputs(handler, tx, cancel).await;
    });
    dbg!("out of launch_send_inputs_task");
    ret
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum InputEvent {
    KB(Keycode),
    MouseButton(MouseButton),
    #[default]
    MouseMovement,
}

/// User event, where the user moves the mouse, presses the mmouse buttons
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, derive_more::TryInto)]
pub enum PotentialInputEvent {
    InputEvent(InputEvent),
    #[default]
    #[try_into(ignore)]
    Dummy,
}

#[derive(Debug, Display, derive_more::From)]
pub struct Timestamp(jiff::Timestamp);

impl Timestamp {
    pub fn now() -> Self {
        let now = jiff::Timestamp::now();
        Self(now)
    }
}

async fn send_inputs(
    handler: &DeviceEventsHandler,
    channel: Sender<(PotentialInputEvent, Timestamp)>,
    cancel: CancellationToken,
) {
    dbg!("Into send_inputs");
    let cloned = channel.clone();
    let mm_guard = setup_mouse_move_handler(handler, cloned);

    let cloned = channel.clone();
    let md_guard = setup_mouse_down_handler(handler, cloned);

    let cloned = channel.clone();
    let kd_guard = setup_key_down_handler(handler, cloned);
    dbg!("starting loop");
    loop {
        tokio::select! {
            _ = cancel.cancelled() => {
                dbg!("send_inputs task cancelled");
                drop(mm_guard);
                drop(md_guard);
                drop(kd_guard);
                break;
            }
            _ = tokio::time::sleep(INPUT_POLL_INTERVAL / 2) =>
            {continue}
        };
    }
    dbg!("Out of send_inputs loop");
}

fn setup_key_down_handler(
    handler: &DeviceEventsHandler,
    tx: Sender<(PotentialInputEvent, Timestamp)>,
) -> CallbackGuard<impl Fn(&Keycode)> {
    handler.on_key_down(move |kd| {
        dbg!("Detected keyboard press");
        tx.send((
            PotentialInputEvent::InputEvent(InputEvent::KB(*kd)),
            Timestamp::now(),
        ))
        .unwrap();
    })
}

fn setup_mouse_down_handler(
    handler: &DeviceEventsHandler,
    tx: Sender<(PotentialInputEvent, Timestamp)>,
) -> CallbackGuard<impl Fn(&usize)> {
    handler.on_mouse_down(move |md| {
        dbg!("Detected mouse key mouvement");
        tx.send((
            PotentialInputEvent::InputEvent(InputEvent::MouseButton(*md)),
            Timestamp::now(),
        ))
        .unwrap();
    })
}

fn setup_mouse_move_handler(
    handler: &DeviceEventsHandler,
    tx: Sender<(PotentialInputEvent, Timestamp)>,
) -> CallbackGuard<impl Fn(&(i32, i32))> {
    handler.on_mouse_move(move |_| {
        dbg!("Detected mouse mouvement");
        tx.send((
            PotentialInputEvent::InputEvent(InputEvent::MouseMovement),
            Timestamp::now(),
        ))
        .unwrap();
    })
}
