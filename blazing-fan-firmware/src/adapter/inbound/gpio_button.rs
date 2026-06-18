use ariel_os::gpio::IntEnabledInput;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};

use crate::core::port::inbound::user_button::UserButton;

pub struct GpioButton<'a, P>
where
    P: UserButton,
{
    input: IntEnabledInput<'a>,
    system: &'a Mutex<CriticalSectionRawMutex, P>,
}

impl<'a, P> GpioButton<'a, P>
where
    P: UserButton,
{
    pub fn new(input: IntEnabledInput<'a>, system: &'a Mutex<CriticalSectionRawMutex, P>) -> Self {
        Self { input, system }
    }

    pub async fn start(&mut self) {
        loop {
            self.input.wait_for_rising_edge().await;
            defmt::info!("BUTTON: Pressed");

            let mut guard = self.system.lock().await;
            guard.on_pressed().await;
        }
    }
}
