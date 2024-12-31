use anyhow::Result;
use core::time::Duration;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{
        peripheral,
        peripherals::Peripherals,
        timer::{self, TimerDriver},
    },
    wifi::{AuthMethod, BlockingWifi, ClientConfiguration, Configuration, EspWifi},
};
use log::info;
use std::error;
use std::{thread::sleep, u8, usize};
use ws2812_esp32_rmt_driver::driver::Ws2812Esp32RmtDriver;

const PRESET_COUNT: usize = 2;
const TIMER_DELAY: u64 = 10;
const LED_COUNT: usize = 18;

enum WifiConnectionError {
    SsidNotProvided(Box<dyn error::Error>),
}

fn setup_wifi(
    ssid: &str,
    pass: &str,
    modem: impl peripheral::Peripheral<P = esp_idf_svc::hal::modem::Modem> + 'static,
    sysloop: EspSystemEventLoop,
) -> Result<Box<EspWifi<'static>>> {
    let mut auth_method = AuthMethod::WPA2Personal;
    if ssid.is_empty() {
        bail!("Missing WiFi name")
    }
    if pass.is_empty() {
        auth_method = AuthMethod::None;
        info!("Wifi password is empty");
    }
    let mut esp_wifi = EspWifi::new(modem, sysloop.clone(), None)?;

    let mut wifi = BlockingWifi::wrap(&mut esp_wifi, sysloop)?;

    wifi.set_configuration(&Configuration::Client(ClientConfiguration::default()))?;

    info!("Starting wifi...");

    wifi.start()?;

    info!("Scanning...");

    let ap_infos = wifi.scan()?;

    let ours = ap_infos.into_iter().find(|a| a.ssid == ssid);

    let channel = if let Some(ours) = ours {
        info!(
            "Found configured access point {} on channel {}",
            ssid, ours.channel
        );
        Some(ours.channel)
    } else {
        info!(
            "Configured access point {} not found during scanning, will go with unknown channel",
            ssid
        );
        None
    };

    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: ssid
            .try_into()
            .expect("Could not parse the given SSID into WiFi config"),
        password: pass
            .try_into()
            .expect("Could not parse the given password into WiFi config"),
        channel,
        auth_method,
        ..Default::default()
    }))?;

    info!("Connecting wifi...");

    wifi.connect()?;

    info!("Waiting for DHCP lease...");

    wifi.wait_netif_up()?;

    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;

    info!("Wifi DHCP info: {:?}", ip_info);

    Ok(Box::new(esp_wifi))
}

enum AudioSource {
    ExternalMicrophone,
    UDPConnection,
}

trait Preset {
    fn id() -> u16;
    fn get_scale_state_count() -> u8;
    fn run(
        led_driver: &mut Ws2812Esp32RmtDriver<'static>,
        timer_driver: &mut TimerDriver<'static>,
        // audio_buffer: Option<Arc<Mutex<AudioSource>>>,
        preset_settings: &PresetSettings,
    ) -> Result<()>;
}

#[derive(Default)]
struct PresetSettings {
    brightness: u8,
    speed: u8,
    scale: u8,
}

struct WifiSettings {
    wifi_ssid: String,
    wifi_password: String,
}

// struct AudioCaptureSettings {
//     audio_source: AudioSource,
//     capture_rate: u32,
// }

struct DeviceSettings {
    wifi_settings: Option<WifiSettings>,
    preset_settings: [PresetSettings; PRESET_COUNT],
    // audio_capture_settings: AudioCaptureSettings,
    current_preset_id: u16,
}

struct Device {
    device_settings: DeviceSettings,
}

struct RunningRainbowPreset {}

impl Preset for RunningRainbowPreset {
    fn id() -> u16 {
        0
    }

    fn get_scale_state_count() -> u8 {
        255
    }

    fn run(
        led_driver: &mut Ws2812Esp32RmtDriver<'static>,
        timer_driver: &mut TimerDriver<'static>,
        // audio_buffer: Option<Arc<Mutex<AudioSource>>>,
        preset_settings: &PresetSettings,
    ) -> Result<()> {
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

fn wheel(wheel_pos: &u8) -> [u8; 3] {
    match wheel_pos {
        0..85 => [wheel_pos * 3, 255 - wheel_pos * 3, 0],
        85..170 => [255 - (wheel_pos - 85) * 3, 0, (wheel_pos - 85) * 3],
        170..=255 => [0, (wheel_pos - 170) * 3, 255 - (wheel_pos - 170) * 3],
    }
}

fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();

    let peripherals = Peripherals::take()?;
    let led_pin = peripherals.pins.gpio13;
    let channel = peripherals.rmt.channel0;
    let mut led_driver = Ws2812Esp32RmtDriver::new(channel, led_pin)?;

    let hardware_timer = peripherals.timer01;
    let timer_config = timer::config::Config::default();
    let mut timer_driver = TimerDriver::new(hardware_timer, &timer_config)?;

    RunningRainbowPreset::run(
        &mut led_driver,
        &mut timer_driver,
        &PresetSettings::default(),
    )?;
    Ok(())
}
