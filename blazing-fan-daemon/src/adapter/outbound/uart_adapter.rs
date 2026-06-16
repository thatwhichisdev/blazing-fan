use blazing_fan_proto::{UART_REQ_MAX_SIZE, UART_RES_MAX_SIZE, UartRequest, UartResponse};
use serial2_tokio::SerialPort;
use tokio::io::AsyncReadExt;

use crate::core::{config::UartConfig, port::outbound::uart_port::UartPort};

pub struct UartAdapter {
    port: SerialPort,
}

impl UartAdapter {
    pub fn new(config: &UartConfig) -> Self {
        let port = SerialPort::open(config.path.clone(), config.baud_rate).unwrap();
        Self { port }
    }
}

impl UartPort for UartAdapter {
    async fn request(&mut self, request: UartRequest) -> Result<UartResponse, ()> {
        let mut tx_buf = [0u8; UART_REQ_MAX_SIZE];
        let mut rx_buf = [0u8; UART_RES_MAX_SIZE];
        let data = postcard::to_slice(&request, &mut tx_buf).unwrap();

        match self.port.write_all(&data).await {
            Ok(()) => match self.port.read_exact(&mut rx_buf).await {
                Ok(_size) => {
                    let response = postcard::from_bytes::<UartResponse>(&rx_buf).unwrap();

                    Ok(response)
                }
                Err(_err) => Err(()),
            },
            Err(_err) => Err(()),
        }
    }
}
