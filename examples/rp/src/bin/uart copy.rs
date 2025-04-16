//! This example shows how to use UART (Universal asynchronous receiver-transmitter) in the RP2040 chip.
//!
//! No specific hardware is specified in this example. Only output on pin 0 is tested.
//! The Raspberry Pi Debug Probe (https://www.raspberrypi.com/products/debug-probe/) could be used
//! with its UART port.

#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_rp::uart;

use {defmt_rtt as _, panic_probe as _};
//use alloc::string::ToString;
use itoa::Buffer;
use heapless::Vec;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let config = uart::Config::default();

    const GREETING: &str = "Hello světe, toto je testovací zpráva č. ";
    const NEWLINE: &str = "\r\n";
    let mut uart: uart::Uart<'_, embassy_rp::peripherals::UART0, uart::Blocking> = uart::Uart::new_with_rtscts_blocking(p.UART0, p.PIN_0, p.PIN_1, p.PIN_3, p.PIN_2, config);
    uart.blocking_write("Hello World!\r\n".as_bytes()).unwrap();
    let mut cnt = 0;
  
    let mut buffer = Buffer::new();
    loop {

        let  counterstr = buffer.format(cnt);
      //  let merged_string = GREETING.to_string() + &stringvalue;

      //let strcnt: String = cnt.to_string().as_str();
   //     let msg= hell + " " + strcnt;
   let mut merged_string: Vec<u8, 20> = Vec::new();
  // let mut merged_string2 = Vec::new();
  if let Err(_) = merged_string.extend_from_slice(GREETING.as_bytes()){ let _line = line!();  uart.blocking_write("ERROR in  merged_string.extend_from_slice(NEWLINE.as_bytes()) \r\n".as_bytes()).unwrap();    };
   if let Err(_) = merged_string.extend_from_slice(counterstr.as_bytes()){ let _line = line!();  uart.blocking_write("ERROR in  merged_string.extend_from_slice(stringvalue.as_bytes()) \r\n".as_bytes()).unwrap();    };
   if let Err(_) = merged_string.extend_from_slice(NEWLINE.as_bytes()) { let _line = line!();  uart.blocking_write("ERROR in  merged_string.extend_from_slice(NEWLINE.as_bytes()) \r\n".as_bytes()).unwrap();    }
        uart.blocking_write(&merged_string).unwrap();
        cnt += 1;
        cortex_m::asm::delay(100_000_000);
    }
    // loop {
    //     uart.blocking_write("hello there2!\r\n".as_bytes()).unwrap();
    //     cortex_m::asm::delay(100_000_000);
    // }
}
