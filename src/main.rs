mod error;
mod presets;
mod wifi;

use bincode;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{
        peripherals::Peripherals,
        timer::{self, TimerDriver},
    },
    nvs::{EspCustomNvs, EspCustomNvsPartition, EspDefaultNvsPartition},
    wifi::EspWifi,
};
use serde::{Deserialize, Serialize};
use wifi::init_wifi;
use ws2812_esp32_rmt_driver::driver::Ws2812Esp32RmtDriver;

pub use crate::error::{Error, Result};
use crate::presets::{Preset, PresetSettings};
use crate::wifi::WiFiSettings;

pub const PRESET_COUNT: usize = 2;
pub const TIMER_DELAY: u64 = 10;
pub const LED_COUNT: usize = 18;
pub const WIFI_AP_SSID: &'static str = "esp-32";
pub const WIFI_AP_PASSWORD: &'static str = "31415926";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct DeviceSettings {
    wifi_settings: WiFiSettings,
    preset_settings: [PresetSettings; PRESET_COUNT],
    current_preset_id: u16,
}

impl DeviceSettings {
    fn save(&self, storage: &mut EspCustomNvs) -> Result<()> {
        storage.set_blob("device_settings", bincode::serialize(self)?.as_slice())?;
        Ok(())
    }

    fn load(storage: &EspCustomNvs) -> Result<Option<DeviceSettings>> {
        let mut buf = [0u8; core::mem::size_of::<DeviceSettings>()];
        match storage.get_blob("device_settings", &mut buf)? {
            Some(_) => Ok(Some(bincode::deserialize(&buf[..])?)),
            None => Ok(None),
        }
    }
}

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();

    let peripherals = Peripherals::take()?;

    let sysloop = EspSystemEventLoop::take()?;

    let default_partition = EspDefaultNvsPartition::take()?;
    let custom_partition = EspCustomNvsPartition::take("device")?;

    let mut storage = EspCustomNvs::new(custom_partition, "device", true)?;

    let settings = DeviceSettings::load(&mut storage)?.unwrap_or(DeviceSettings::default());

    let mut wifi_driver =
        EspWifi::new(peripherals.modem, sysloop.clone(), Some(default_partition))?;

    init_wifi(&mut wifi_driver, sysloop, &settings.wifi_settings)?;

    let led_pin = peripherals.pins.gpio13;
    let channel = peripherals.rmt.channel0;
    let mut led_driver = Ws2812Esp32RmtDriver::new(channel, led_pin)?;

    let hardware_timer = peripherals.timer00;
    let timer_config = timer::config::Config::default();
    let mut timer_driver = TimerDriver::new(hardware_timer, &timer_config)?;

    crate::presets::running_rainbow::RunningRainbowPreset::run(
        &mut led_driver,
        &mut timer_driver,
        &PresetSettings::default(),
    )?;

    Ok(())
}
