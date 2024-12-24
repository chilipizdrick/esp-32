use anyhow::Result;
use core::time::Duration;
use esp_idf_svc::hal::{gpio::PinDriver, peripherals::Peripherals};
use std::thread::sleep;

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();

    let peripherals = Peripherals::take()?;
    let mut driver = PinDriver::output(peripherals.pins.gpio2)?;

    loop {
        sleep(Duration::from_millis(500));
        driver.set_high()?;
        sleep(Duration::from_millis(500));
        driver.set_low()?;
    }
}
