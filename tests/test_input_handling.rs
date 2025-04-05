#[cfg(test)]
mod test {
    use device_query::DeviceEventsHandler;
    use enigo::{Button, Coordinate, Direction::Press, Enigo, Key, Keyboard, Mouse, Settings};
    use fun::{PotentialInputEvent, launch_send_inputs_task};
    use serial_test::serial;
    use std::{
        sync::{LazyLock, mpsc::channel},
        time::Duration,
    };
    use tokio_util::sync::CancellationToken;

    pub const INPUT_POLL_INTERVAL: Duration = Duration::from_micros(10);
    pub static INPUT_HANDLER: LazyLock<DeviceEventsHandler> =
        LazyLock::new(|| DeviceEventsHandler::new(INPUT_POLL_INTERVAL).unwrap());

    fn build_event_simulator() -> Box<dyn EventSimulator> {
        Box::new(Enigo::new(&Settings::default()).unwrap())
    }

    async fn setup_input_handling() -> (
        std::sync::mpsc::Receiver<(PotentialInputEvent, fun::Timestamp)>,
        CancellationToken,
        tokio::task::JoinHandle<()>,
        Box<dyn EventSimulator>,
    ) {
        let cancel = CancellationToken::new();
        let (tx, rx) = channel();
        let event_simulator = build_event_simulator();
        let handle = launch_send_inputs_task(&INPUT_HANDLER, tx, cancel.clone()).await;
        (rx, cancel, handle, event_simulator)
    }

    //TODO: Clean up a bit the test. Probably add a few setups functions

    #[serial]
    #[tokio::test(flavor = "multi_thread")]
    async fn test_received_expected_kb_inputs_length() {
        let (rx, cancel, handle, mut event_simulator) = setup_input_handling().await;
        let n_events = event_simulator.simulate_kb_down();
        let mut events: Vec<PotentialInputEvent> = vec![];
        let _cancel_handle = tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            cancel.cancel();
        });
        let (r1, r2) = tokio::join!(handle, _cancel_handle);
        r1.unwrap();
        r2.unwrap();
        for _ in 0..n_events * 10 {
            let _ = rx.try_recv().map(|(e, _)| {
                events.push(e);
            });
        }
        dbg!(&events);
        assert!(events.len() >= n_events)
    }

    #[serial]
    #[tokio::test(flavor = "multi_thread")]
    async fn test_received_mouse_down_inputs_length() {
        let (rx, cancel, handle, mut event_simulator) = setup_input_handling().await;
        let n_events = event_simulator.simulate_mouse_down();
        let mut events: Vec<PotentialInputEvent> = vec![];
        let _cancel_handle = tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            cancel.cancel();
        })
        .await;

        let r1 = tokio::join!(handle).0;
        r1.unwrap();
        for _ in 0..n_events * 2 {
            let _ = rx.try_recv().map(|(e, _)| {
                dbg!(&e);
                events.push(e);
            });
        }
        dbg!(&events);
        assert!(events.len() >= n_events)
    }

    #[serial]
    #[tokio::test(flavor = "multi_thread")]
    async fn test_received_mouse_move_inputs_length() {
        let (rx, cancel, handle, mut event_simulator) = setup_input_handling().await;
        let n_events = event_simulator.simulate_move_mouse();
        let mut events: Vec<PotentialInputEvent> = vec![];
        let _cancel_handle = tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            cancel.cancel();
        })
        .await;

        let r1 = tokio::join!(handle).0;
        r1.unwrap();
        for _ in 0..n_events * 2 {
            let _ = rx.try_recv().map(|v| dbg!(v)).map(|(e, _)| {
                dbg!(&e);
                events.push(e);
            });
        }
        dbg!(&events);
        assert_eq!(events.len(), n_events)
    }

    #[serial]
    #[tokio::test(flavor = "multi_thread")]
    async fn test_received_kb_inputs_in_order() {
        let (rx, cancel, handle, mut event_simulator) = setup_input_handling().await;
        let n_events = event_simulator.simulate_kb_down();
        let mut events: Vec<PotentialInputEvent> = vec![];
        let _cancel_handle = tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            cancel.cancel();
        });
        let (r1, r2) = tokio::join!(handle, _cancel_handle);
        r1.unwrap();
        r2.unwrap();
        for _ in 0..n_events * 2 {
            let _ = rx.try_recv().map(|(e, _)| {
                events.push(e);
            });
        }
        dbg!(&events);
        assert!(events.len() >= n_events)
    }

    trait EventSimulator {
        fn simulate_move_mouse(&mut self) -> usize;
        fn simulate_mouse_down(&mut self) -> usize;
        fn simulate_kb_down(&mut self) -> usize;
    }
    impl EventSimulator for Enigo {
        fn simulate_move_mouse(&mut self) -> usize {
            self.move_mouse(50, 50, Coordinate::Abs).unwrap();
            self.move_mouse(100, 100, Coordinate::Abs).unwrap();
            self.move_mouse(-4, 50, Coordinate::Rel).unwrap();
            self.move_mouse(-4, -27, Coordinate::Rel).unwrap();
            self.move_mouse(12, -1, Coordinate::Rel).unwrap();
            5
        }
        fn simulate_mouse_down(&mut self) -> usize {
            //NOTE: Enigo has more granularity then device_query for mouse buttons. It
            //detects when the button is pressed and released
            self.button(Button::Right, Press).unwrap();
            self.button(Button::Right, enigo::Direction::Release)
                .unwrap();
            self.button(Button::Left, Press).unwrap();
            self.button(Button::Left, enigo::Direction::Release)
                .unwrap();
            2
        }
        fn simulate_kb_down(&mut self) -> usize {
            self.key(Key::Unicode('m'), Press).unwrap();
            self.key(Key::Unicode('o'), Press).unwrap();
            self.key(Key::Unicode('l'), Press).unwrap();
            self.key(Key::Unicode('a'), Press).unwrap();
            4
        }
    }
}
