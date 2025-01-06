mod error;
mod presets;
mod wifi;

use core::time::Duration;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{
        peripherals::Peripherals,
        timer::{self, TimerDriver},
    },
    nvs::{EspCustomNvs, EspCustomNvsPartition, EspDefaultNvsPartition},
    wifi::EspWifi,
};
use std::{
    ffi::{CStr, CString},
    os::raw::c_char,
    thread::sleep,
    u8, usize,
};
use wifi::create_wifi_ap;
use ws2812_esp32_rmt_driver::driver::Ws2812Esp32RmtDriver;

pub use crate::error::{Error, Result};

const PRESET_COUNT: usize = 2;
const TIMER_DELAY: u64 = 10;
const LED_COUNT: usize = 18;
const WIFI_AP_SSID: &str = "esp-32";
const WIFI_AP_PASSWORD: &str = "31415926";

trait Preset {
    fn get_scale_state_count() -> u8;
    fn run(
        led_driver: &mut Ws2812Esp32RmtDriver<'static>,
        timer_driver: &mut TimerDriver<'static>,
        preset_settings: &PresetSettings,
    ) -> Result<()>;
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
struct PresetSettings {
    brightness: u8,
    speed: u8,
    scale: u8,
}

#[repr(C)]
#[derive(Debug, Clone)]
struct WifiSettings {
    wifi_ssid: [c_char; 32],
    wifi_password: [c_char; 64],
}

#[repr(C)]
#[derive(Debug, Clone)]
struct DeviceSettings {
    wifi_settings: Option<WifiSettings>,
    preset_settings: [PresetSettings; PRESET_COUNT],
    current_preset_id: u16,
}

struct Device {
    device_settings: DeviceSettings,
}

struct RunningRainbowPreset {}

impl Preset for RunningRainbowPreset {
    fn get_scale_state_count() -> u8 {
        255
    }

    fn run(
        led_driver: &mut Ws2812Esp32RmtDriver<'static>,
        timer_driver: &mut TimerDriver<'static>,
        _preset_settings: &PresetSettings,
    ) -> Result<()> {
        fn wheel(wheel_pos: &u8) -> [u8; 3] {
            match wheel_pos {
                0..85 => [wheel_pos * 3, 255 - wheel_pos * 3, 0],
                85..170 => [255 - (wheel_pos - 85) * 3, 0, (wheel_pos - 85) * 3],
                170..=255 => [0, (wheel_pos - 170) * 3, 255 - (wheel_pos - 170) * 3],
            }
        }

        let timer_tick_hz = timer_driver.tick_hz();
        let frame_ticks = TIMER_DELAY * timer_tick_hz / 1000;
        let mut strip: Vec<u8> = vec![0; LED_COUNT * 3];
        let mut pixel: [u8; 3];

        timer_driver.enable(true)?;
        timer_driver.set_counter(0)?;

        loop {
            for i in 0..256 as usize {
                for j in 0..LED_COUNT as usize {
                    let wheel_pos = (((j * 256 / LED_COUNT) + i) % 256) as u8;
                    pixel = wheel(&wheel_pos);
                    (0..3).for_each(|idx| {
                        strip[3 * j + idx] = pixel[idx];
                    });
                }
                while timer_driver.counter()? < frame_ticks {
                    sleep(Duration::from_millis(1));
                }
                timer_driver.set_counter(0)?;
                led_driver.write_blocking(strip.clone().into_iter())?;
            }
        }
    }
}

unsafe fn into_u8_slice<T: Sized>(data: &T) -> &[u8] {
    core::slice::from_raw_parts((data as *const T) as *const u8, core::mem::size_of::<T>())
}

unsafe fn from_u8_slice<T: Sized>(data: &[u8]) -> T {
    let (head, body, _tail) = (*data).align_to::<T>();
    assert!(head.is_empty(), "Data was not aligned");
    body
}

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();

    let peripherals = Peripherals::take()?;
    let sysloop = EspSystemEventLoop::take()?;

    let custom_partition = EspCustomNvsPartition::take("device")?;

    let storage = EspCustomNvs::new(custom_partition, "device", true)?;

    let raw_device_config: [u8; core::mem::size_of::<DeviceSettings>()];

    storage.get_raw("device_config", buf)?;

    let default_partition = EspDefaultNvsPartition::take()?;

    let mut wifi_driver =
        EspWifi::new(peripherals.modem, sysloop.clone(), Some(default_partition))?;

    create_wifi_ap(&mut wifi_driver, sysloop, WIFI_AP_SSID, WIFI_AP_PASSWORD)?;

    let led_pin = peripherals.pins.gpio13;
    let channel = peripherals.rmt.channel0;
    let mut led_driver = Ws2812Esp32RmtDriver::new(channel, led_pin)?;

    let hardware_timer = peripherals.timer00;
    let timer_config = timer::config::Config::default();
    let mut timer_driver = TimerDriver::new(hardware_timer, &timer_config)?;

    RunningRainbowPreset::run(
        &mut led_driver,
        &mut timer_driver,
        &PresetSettings::default(),
    )?;

    Ok(())
}
