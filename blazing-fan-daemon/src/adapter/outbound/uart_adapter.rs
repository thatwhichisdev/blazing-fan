use blazing_fan_proto::{
    Frame, FrameBody, FrameHeader, REQUEST_MAX_SIZE, RESPONSE_MAX_SIZE, UartRequest, UartResponse,
};
use serial2_tokio::{CharSize, FlowControl, Parity, SerialPort, Settings, StopBits};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::core::{
    config::UartConfig,
    port::outbound::uart_port::{UartError, UartPort},
};

pub struct UartAdapter {
    port: SerialPort,
}

impl UartAdapter {
    pub fn new(config: &UartConfig) -> Result<Self, UartError> {
        let port = SerialPort::open("/dev/ttyAMA4", |mut settings: Settings| {
            settings.set_raw();
            settings.set_baud_rate(config.baud_rate)?;
            settings.set_char_size(CharSize::Bits8);
            settings.set_stop_bits(StopBits::One);
            settings.set_parity(Parity::None);
            settings.set_flow_control(FlowControl::None);
            Ok(settings)
        })?;

        Ok(Self { port })
    }

    async fn read_frame(&mut self) -> Result<Frame<RESPONSE_MAX_SIZE>, UartError> {
        let mut header_raw = [0u8; 2];
        self.port.read(&mut header_raw).await?;
        let header = FrameHeader::from(header_raw);

        let mut body_raw = [0u8; RESPONSE_MAX_SIZE];
        let body_len = usize::from(header.length);
        self.port.read_exact(&mut body_raw[..body_len]).await?;

        let body = FrameBody::<RESPONSE_MAX_SIZE>::from_slice(&body_raw)?;

        Ok(Frame { header, body })
    }

    async fn write_frame(&mut self, frame: Frame<REQUEST_MAX_SIZE>) -> Result<(), UartError> {
        tracing::info!("writing header {:?}", &frame.header.as_slice());
        self.port.write_all(&frame.header.as_slice()).await?;
        tracing::info!("writing body {:?}", &frame.body.as_slice());
        self.port.write_all(frame.body.as_slice()).await?;
        self.port.flush().await?;

        Ok(())
    }
}

impl UartPort for UartAdapter {
    async fn request(&mut self, request: UartRequest) -> Result<UartResponse, UartError> {
        let request_frame = Frame::<REQUEST_MAX_SIZE>::try_from(&request)?;
        self.write_frame(request_frame).await?;

        let response_frame = self.read_frame().await?;
        let response = UartResponse::try_from(&response_frame)?;

        Ok(response)
    }

    async fn shutdown(&mut self) -> Result<(), UartError> {
        self.port.discard_buffers()?;
        self.port.shutdown().await?;

        Ok(())
    }
}
