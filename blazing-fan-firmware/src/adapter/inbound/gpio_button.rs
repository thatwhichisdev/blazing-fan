use ariel_os::gpio::IntEnabledInput;
use async_button::{Button, ButtonConfig};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};

use crate::core::port::inbound::user_button::UserButton;

pub struct GpioButton<'a, P>
where
    P: UserButton,
{
    button: Button<IntEnabledInput<'a>>,
    system: &'a Mutex<CriticalSectionRawMutex, P>,
}

impl<'a, P> GpioButton<'a, P>
where
    P: UserButton,
{
    pub fn new(button: IntEnabledInput<'a>, system: &'a Mutex<CriticalSectionRawMutex, P>) -> Self {
        let button = Button::new(button, ButtonConfig::default());
        Self { button, system }
    }

    pub async fn start(&mut self) {
        loop {
            match self.button.update().await {
                async_button::ButtonEvent::ShortPress { count } => {
                    defmt::info!("BUTTON: User pressed {=usize} times in the row", count);
                }
                async_button::ButtonEvent::LongPress => {
                    defmt::info!("BUTTON: User made long press");
                    let mut guard = self.system.lock().await;
                    guard.on_pressed().await;
                }
            }
        }
    }
}
