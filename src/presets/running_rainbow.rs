use esp_idf_svc::hal::timer::TimerDriver;
use std::thread::sleep;
use std::time::Duration;
use ws2812_esp32_rmt_driver::Ws2812Esp32RmtDriver;

use crate::presets::Preset;
use crate::PresetSettings;
use crate::Result;
use crate::LED_COUNT;
use crate::TIMER_DELAY;

pub struct RunningRainbowPreset {}

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
