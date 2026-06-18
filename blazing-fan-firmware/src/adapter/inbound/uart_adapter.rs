use crate::core::port::inbound::uart_port::{UartError, UartPort};
use ariel_os::hal::uart;
use blazing_fan_proto::{
    FanError, UART_REQ_MAX_SIZE, UART_RES_MAX_SIZE, UartRequest, UartResponse,
};
use defmt::{error, info};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use embedded_io_async::{Read as _, Write as _};

#[derive(Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub enum UartName {
    A,
    B,
}

impl defmt::Format for UartName {
    fn format(&self, fmt: defmt::Formatter) {
        match self {
            UartName::A => defmt::write!(fmt, "UART_A"),
            UartName::B => defmt::write!(fmt, "UART_B"),
        }
    }
}

pub struct UartAdapter<'a, P>
where
    P: UartPort,
{
    uart: uart::Uart<'a>,
    rx_buf: &'a mut [u8; UART_REQ_MAX_SIZE],
    tx_buf: &'a mut [u8; UART_RES_MAX_SIZE],
    port: &'a Mutex<CriticalSectionRawMutex, P>,
    name: UartName,
}

impl<'a, P> UartAdapter<'a, P>
where
    P: UartPort,
{
    pub fn new(
        uart: uart::Uart<'a>,
        rx_buf: &'a mut [u8; UART_REQ_MAX_SIZE],
        tx_buf: &'a mut [u8; UART_RES_MAX_SIZE],
        port: &'a Mutex<CriticalSectionRawMutex, P>,
        name: UartName,
    ) -> Self {
        Self {
            uart,
            rx_buf,
            tx_buf,
            port,
            name,
        }
    }

    pub async fn start(&mut self) {
        defmt::info!("{=?}: Listener started", self.name);

        loop {
            if let Err(err) = self.uart.read_exact(self.rx_buf).await {
                error!(
                    "{=?}: Failed to read data from port [err: {=?}]",
                    self.name,
                    defmt::Debug2Format(&err)
                );
                continue;
            };

            info!(
                "{=?}: Read data from UART0 port [data: {=[u8]}]",
                self.name,
                self.rx_buf.as_slice()
            );

            let request = match postcard::from_bytes::<UartRequest>(&self.rx_buf[..]) {
                Ok(request) => request,
                Err(err) => {
                    defmt::error!(
                        "{}: Failed to deserialize UART request [err: {=?}]",
                        self.name,
                        defmt::Debug2Format(&err)
                    );

                    let response = UartResponse::Error(FanError::InvalidRequest);
                    let data = postcard::to_slice(&response, self.tx_buf).unwrap();
                    self.uart.write(data).await.unwrap();
                    self.uart.flush().await.unwrap();

                    continue;
                }
            };

            let mut guard = self.port.lock().await;

            match guard.request(request, &self.name).await {
                Ok(response) => {
                    let data = postcard::to_slice(&response, self.tx_buf).unwrap();
                    self.uart.write(data).await.unwrap();
                    self.uart.flush().await.unwrap();
                }
                Err(e) => {
                    let response = match e {
                        UartError::EmcError(emc_err) => {
                            defmt::error!(
                                "{}: emc internal error [err: {=?}]",
                                self.name,
                                defmt::Debug2Format(&emc_err)
                            );

                            UartResponse::Error(FanError::EmcError)
                        }
                        UartError::RpError(mcu_err) => {
                            defmt::error!(
                                "{}: mcu internal error [err: {=?}]",
                                self.name,
                                defmt::Debug2Format(&mcu_err)
                            );

                            UartResponse::Error(FanError::McuError)
                        }
                    };

                    let data = postcard::to_slice(&response, self.tx_buf).unwrap();
                    self.uart.write(data).await.unwrap();
                    self.uart.flush().await.unwrap();
                }
            }
        }
    }
}
