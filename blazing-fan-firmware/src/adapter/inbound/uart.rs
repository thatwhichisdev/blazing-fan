use crate::core::port::inbound::uart_port::{UartError, UartName, UartPort};

use ariel_os::hal::uart;
use blazing_fan_proto::{
    Frame, FrameBody, FrameHeader, REQUEST_MAX_SIZE, RESPONSE_MAX_SIZE, UartRequest,
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use embedded_io_async::{Read as _, Write as _};

pub struct UartAdapter<'a, P>
where
    P: UartPort,
{
    uart: uart::Uart<'a>,
    #[allow(unused)]
    rx_buf: &'a mut [u8; REQUEST_MAX_SIZE],
    #[allow(unused)]
    tx_buf: &'a mut [u8; RESPONSE_MAX_SIZE],
    core: &'a Mutex<CriticalSectionRawMutex, P>,
    name: UartName,
}

impl<'a, P> UartAdapter<'a, P>
where
    P: UartPort,
{
    pub fn new(
        uart: uart::Uart<'a>,
        rx_buf: &'a mut [u8; REQUEST_MAX_SIZE],
        tx_buf: &'a mut [u8; RESPONSE_MAX_SIZE],
        core: &'a Mutex<CriticalSectionRawMutex, P>,
        name: UartName,
    ) -> Self {
        Self {
            uart,
            rx_buf,
            tx_buf,
            core,
            name,
        }
    }

    pub async fn start(&mut self) {
        defmt::info!("{=?}: Listener started", self.name);

        loop {
            let request_frame = match self.read_frame().await {
                Ok(frame) => frame,
                Err(e) => {
                    defmt::error!(
                        "{}: failed to read the frame {=?}",
                        self.name,
                        defmt::Debug2Format(&e)
                    );
                    continue;
                }
            };

            let request = match UartRequest::try_from(&request_frame) {
                Ok(request) => request,
                Err(e) => {
                    defmt::error!(
                        "{}: failed to deserealize the frame {=?}",
                        self.name,
                        defmt::Debug2Format(&e)
                    );
                    continue;
                }
            };

            let mut core = self.core.lock().await;
            let response = match core.request(request).await {
                Ok(response) => response,
                Err(e) => {
                    defmt::error!(
                        "{}: error happened while processing the request {=?}",
                        self.name,
                        defmt::Debug2Format(&e)
                    );
                    continue;
                }
            };
            drop(core);

            let response_frame = match Frame::<RESPONSE_MAX_SIZE>::try_from(&response) {
                Ok(frame) => frame,
                Err(e) => {
                    defmt::error!(
                        "{}: failed to serealize the frame {=?}",
                        self.name,
                        defmt::Debug2Format(&e)
                    );
                    continue;
                }
            };

            if let Err(e) = self.write_frame(response_frame).await {
                defmt::error!(
                    "{}: failed to write the frame {=?}",
                    self.name,
                    defmt::Debug2Format(&e)
                );
            };
        }
    }

    async fn read_frame(&mut self) -> Result<Frame<REQUEST_MAX_SIZE>, UartError> {
        let mut header_raw = [0u8; 2];
        self.uart.read_exact(&mut header_raw).await?;
        let header: FrameHeader = header_raw.into();
        defmt::info!("read header {:?}", header);

        let mut body_raw = [0u8; REQUEST_MAX_SIZE];
        let body_len = usize::from(header.length);
        self.uart.read_exact(&mut body_raw[..body_len]).await?;
        let body: FrameBody<REQUEST_MAX_SIZE> = FrameBody::from_slice(&body_raw[..body_len])?;
        defmt::info!("read body {:?}", body);

        Ok(Frame { header, body })
    }

    async fn write_frame(&mut self, frame: Frame<RESPONSE_MAX_SIZE>) -> Result<(), UartError> {
        self.uart.write_all(&frame.header.as_slice()).await?;
        self.uart.write_all(frame.body.as_slice()).await?;
        self.uart.flush().await?;

        Ok(())
    }
}
