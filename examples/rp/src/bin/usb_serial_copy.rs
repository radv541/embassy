#![no_std]
#![no_main]

use core::str;
use defmt::{info, panic, unwrap};
use embassy_executor::Spawner;
use embassy_rp::bind_interrupts;
use embassy_rp::peripherals::USB;
use embassy_rp::usb::{Driver, Instance, InterruptHandler};
use embassy_usb::class::cdc_acm::{CdcAcmClass, State};
use embassy_usb::driver::EndpointError;
use embassy_usb::UsbDevice;
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Calculator Dialog Example!");

    let p = embassy_rp::init(Default::default());

    // Create the driver
    let driver = Driver::new(p.USB, Irqs);

    // Create USB configuration
    let config = {
        let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
        config.manufacturer = Some("Embassy");
        config.product = Some("Calculator Dialog");
        config.serial_number = Some("12345678");
        config.max_power = 100;
        config.max_packet_size_0 = 64;
        config
    };

    // Build USB device
    let mut builder = {
        static CONFIG_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
        static BOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
        static CONTROL_BUF: StaticCell<[u8; 64]> = StaticCell::new();

        embassy_usb::Builder::new(
            driver,
            config,
            CONFIG_DESCRIPTOR.init([0; 256]),
            BOS_DESCRIPTOR.init([0; 256]),
            &mut [],
            CONTROL_BUF.init([0; 64]),
        )
    };

    // Create CDC ACM class
    let mut class = {
        static STATE: StaticCell<State> = StaticCell::new();
        let state = STATE.init(State::new());
        CdcAcmClass::new(&mut builder, state, 64)
    };

    // Build the USB device
    let usb = builder.build();

    // Run USB task
    unwrap!(spawner.spawn(usb_task(usb)));

    // Calculator dialog loop
    loop {
        class.wait_connection().await;
        info!("Connected");
        let _ = calculator_dialog(&mut class).await;
        info!("Disconnected");
    }
}

type MyUsbDriver = Driver<'static, USB>;
type MyUsbDevice = UsbDevice<'static, MyUsbDriver>;

#[embassy_executor::task]
async fn usb_task(mut usb: MyUsbDevice) -> ! {
    usb.run().await
}

struct Disconnected {}

impl From<EndpointError> for Disconnected {
    fn from(val: EndpointError) -> Self {
        match val {
            EndpointError::BufferOverflow => panic!("Buffer overflow"),
            EndpointError::Disabled => Disconnected {},
        }
    }
}

async fn calculator_dialog<'d, T: Instance + 'd>(
    class: &mut CdcAcmClass<'d, Driver<'d, T>>,
) -> Result<(), Disconnected> {
    let mut buf = [0; 64];
    
    loop {
        class.write_packet(b"Enter calculation (e.g., 3+4): ").await?;
        
        // Read user input
        let n = class.read_packet(&mut buf).await?;
        let input_str = str::from_utf8(&buf[..n]).unwrap_or("");
        
        if input_str.trim().is_empty() {
            continue;
        }
        
        // Parse and calculate result
        match parse_and_calculate(input_str.trim()) {
            Ok(result) => {
                let response = format!("Result: {}\r\n", result);
                class.write_packet(response.as_bytes()).await?;
            }
            Err(e) => {
                let response = format!("Error: {}\r\n", e);
                class.write_packet(response.as_bytes()).await?;
            }
        }
    }
}

// Function to parse and calculate basic arithmetic expressions
fn parse_and_calculate(input: &str) -> Result<f32, &'static str> {
    let parts: Vec<&str> = input.split(|c| c == '+' || c == '-' || c == '*' || c == '/').collect();
    
    if parts.len() != 2 {
        return Err("Invalid format. Use <num1><operator><num2>");
    }

    let num1: f32 = parts[0].trim().parse().map_err(|_| "Invalid number")?;
    let num2: f32 = parts[1].trim().parse().map_err(|_| "Invalid number")?;

    if input.contains('+') {
        Ok(num1 + num2)
    } else if input.contains('-') {
        Ok(num1 - num2)
    } else if input.contains('*') {
        Ok(num1 * num2)
    } else if input.contains('/') {
        if num2 == 0.0 {
            Err("Division by zero")
        } else {
            Ok(num1 / num2)
        }
    } else {
        Err("Unknown operator")
    }
}