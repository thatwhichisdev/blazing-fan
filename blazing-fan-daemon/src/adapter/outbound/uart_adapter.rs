use std::time::Duration;

use blazing_fan_proto::{UART_REQ_MAX_SIZE, UART_RES_MAX_SIZE, UartRequest, UartResponse};
use crc::{CRC_32_ISCSI, Crc};
use serial2_tokio::SerialPort;
use tokio::io::AsyncWriteExt;

use crate::core::{
    config::UartConfig,
    port::outbound::uart_port::{UartError, UartPort},
};

const CRC32: Crc<u32> = Crc::<u32>::new(&CRC_32_ISCSI);

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
        let mut tx_buf = [0u8; UART_REQ_MAX_SIZE + 4];
        let mut rx_buf = [0u8; UART_RES_MAX_SIZE + 4];
        let data = postcard::to_slice(&request, &mut tx_buf)?;

        match self.port.write_all(data).await {
            Ok(()) => {
                self.port.flush().await.unwrap();

                tokio::select! {
                    res = self.port.read(&mut rx_buf) => {
                        match res {
                            Ok(_size) => {
                                let response = postcard::from_bytes_crc32::<UartResponse>(&rx_buf, CRC32.digest())?;

                                Ok(response)
                            }
                            Err(e) => Err(UartError::IoError(e)),
                        }
                    }
                    () = tokio::time::sleep(Duration::from_secs(10)) => {
                        Err(UartError::Timeout)
                    }
                }
            }
            Err(e) => Err(UartError::IoError(e)),
        }
    }

    async fn shutdown(&mut self) -> Result<(), UartError> {
        self.port.discard_buffers()?;
        self.port.shutdown().await?;

        Ok(())
    }
}
