use crate::ports::button_port::ButtonPort;
use ariel_os::gpio::IntEnabledInput;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};

pub struct ButtonAdapter<'a, P>
where
    P: ButtonPort,
{
    btn: IntEnabledInput<'a>,
    port: &'a Mutex<CriticalSectionRawMutex, P>,
}

impl<'a, P> ButtonAdapter<'a, P>
where
    P: ButtonPort,
{
    pub fn new(btn: IntEnabledInput<'a>, port: &'a Mutex<CriticalSectionRawMutex, P>) -> Self {
        Self { btn, port }
    }

    pub async fn start(&mut self) {
        loop {
            self.btn.wait_for_rising_edge().await;
            defmt::info!("BUTTON: Pressed");

            let mut guard = self.port.lock().await;
            guard.btn_pressed().await;
        }
    }
}
