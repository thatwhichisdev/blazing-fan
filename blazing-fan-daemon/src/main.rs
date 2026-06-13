use blazing_fan_proto::{UART_RES_MAX_SIZE, UartResponse};
use serial2_tokio::SerialPort;
use tokio::io::AsyncReadExt;

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    let mut port = SerialPort::open("/dev/ttyACM5", 115200).unwrap();

    loop {
        let mut rx_buf = [0u8; UART_RES_MAX_SIZE];
        let Ok(_size) = port.read_exact(&mut rx_buf).await else {
            continue;
        };

        let response = postcard::from_bytes::<UartResponse>(&rx_buf[..]).unwrap();
        match response {
            UartResponse::Ok => todo!(),
            UartResponse::Err => todo!(),
            UartResponse::Status { .. } => todo!(),
        }
    }
}
