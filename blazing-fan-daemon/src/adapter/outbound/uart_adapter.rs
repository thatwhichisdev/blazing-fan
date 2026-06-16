use std::time::Duration;

use blazing_fan_proto::{UART_REQ_MAX_SIZE, UART_RES_MAX_SIZE, UartRequest, UartResponse};
use serial2_tokio::SerialPort;
use tokio::io::AsyncReadExt;

use crate::core::{
    config::UartConfig,
    port::outbound::uart_port::{UartError, UartPort},
};

pub struct UartAdapter {
    port: SerialPort,
}

impl UartAdapter {
    pub fn new(config: &UartConfig) -> Result<Self, UartError> {
        match SerialPort::open(config.path.clone(), config.baud_rate) {
            Ok(port) => Ok(Self { port }),
            Err(e) => Err(UartError::IoError(e)),
        }
    }
}

impl UartPort for UartAdapter {
    async fn request(&mut self, request: UartRequest) -> Result<UartResponse, UartError> {
        let mut tx_buf = [0u8; UART_REQ_MAX_SIZE];
        let mut rx_buf = [0u8; UART_RES_MAX_SIZE];
        let data = postcard::to_slice(&request, &mut tx_buf).map_err(UartError::PostcardError)?;

        match self.port.write_all(data).await {
            Ok(()) => {
                tokio::select! {
                    // read branch
                    res = self.port.read_exact(&mut rx_buf) => {
                        match res {
                            Ok(_size) => {
                                let response = postcard::from_bytes::<UartResponse>(&rx_buf).map_err(UartError::PostcardError)?;

                                Ok(response)
                            }
                            Err(e) => Err(UartError::IoError(e)),
                        }
                    }
                    // timout branch
                    () = tokio::time::sleep(Duration::from_secs(1)) => {
                        Err(UartError::Timeout)
                    }
                }
            }
            Err(e) => Err(UartError::IoError(e)),
        }
    }
}
