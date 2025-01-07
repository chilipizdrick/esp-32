use esp_idf_svc::hal::timer::TimerDriver;
use serde::{Deserialize, Serialize};
use ws2812_esp32_rmt_driver::Ws2812Esp32RmtDriver;

pub mod running_rainbow;
use crate::Result;

pub trait Preset {
    fn get_scale_state_count() -> u8;
    fn run(
        led_driver: &mut Ws2812Esp32RmtDriver<'static>,
        timer_driver: &mut TimerDriver<'static>,
        preset_settings: &PresetSettings,
    ) -> Result<()>;
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct PresetSettings {
    brightness: u8,
    speed: u8,
    scale: u8,
}

impl Default for PresetSettings {
    fn default() -> Self {
        Self {
            brightness: u8::MAX / 2,
            speed: u8::MAX / 2,
            scale: u8::MAX / 2,
        }
    }
}

pub fn get_preset_by_id(id: &u16) -> Option<impl Preset> {
    match id {
        0 => Some(self::running_rainbow::RunningRainbowPreset {}),
        _ => None,
    }
}
