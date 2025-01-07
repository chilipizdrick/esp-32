use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    wifi::{
        AccessPointConfiguration, AuthMethod, BlockingWifi, ClientConfiguration, Configuration,
        EspWifi,
    },
};
use serde::{Deserialize, Serialize};

use crate::Result;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum WiFiMode {
    Client,
    Server,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WiFiSettings {
    mode: WiFiMode,
    ssid: String,
    password: String,
}

impl Default for WiFiSettings {
    fn default() -> Self {
        Self {
            ssid: crate::WIFI_AP_SSID.to_string(),
            password: crate::WIFI_AP_PASSWORD.to_string(),
            mode: WiFiMode::Server,
        }
    }
}

pub fn create_wifi_ap(
    driver: &mut EspWifi<'static>,
    sysloop: EspSystemEventLoop,
    ssid: &str,
    password: &str,
) -> Result<()> {
    let mut auth_method = AuthMethod::WPA2Personal;
    if ssid.is_empty() {
        return Err("Missing SSID".into());
    }
    if password.is_empty() {
        auth_method = AuthMethod::None;
        log::info!("Setting up AP with no password");
    }

    let mut wifi = BlockingWifi::wrap(driver, sysloop)?;

    wifi.set_configuration(&esp_idf_svc::wifi::Configuration::AccessPoint(
        AccessPointConfiguration {
            ssid: ssid.try_into().map_err(|_| "Error parising SSID")?,
            password: password.try_into().map_err(|_| "Error parising password")?,
            auth_method,
            ..Default::default()
        },
    ))?;

    log::info!("Starting wifi...");

    wifi.start()?;

    Ok(())
}

pub fn connect_to_wifi_ap(
    driver: &mut EspWifi<'static>,
    sysloop: EspSystemEventLoop,
    ssid: &str,
    password: &str,
) -> Result<()> {
    let mut auth_method = AuthMethod::WPA2Personal;
    if ssid.is_empty() {
        return Err("Missing SSID".into());
    }
    if password.is_empty() {
        auth_method = AuthMethod::None;
        log::info!("Wifi password is empty. Using AuthMethond::None");
    }

    let mut wifi = BlockingWifi::wrap(driver, sysloop)?;

    wifi.set_configuration(&esp_idf_svc::wifi::Configuration::Client(
        ClientConfiguration::default(),
    ))?;

    log::info!("Starting wifi...");

    wifi.start()?;

    log::info!("Scanning...");

    let ap_infos = wifi.scan()?;

    let ap = ap_infos.into_iter().find(|ap| ap.ssid == ssid);

    let channel = match ap {
        Some(ap) => {
            log::info!(
                "Found configured access point {} on channel {}",
                ssid,
                ap.channel
            );
            Some(ap.channel)
        }
        None => {
            log::info!(
                "Configured access point {} not found during scanning, proceeding unknown channel",
                ssid
            );
            None
        }
    };

    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: ssid.try_into().map_err(|_| "Error parsing SSID")?,
        password: password.try_into().map_err(|_| "Error parsing password")?,
        channel,
        auth_method,
        ..Default::default()
    }))?;

    log::info!("Connecting to wifi access point...");

    wifi.connect()?;

    log::info!("Waiting for DHCP lease...");

    wifi.wait_netif_up()?;

    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;

    log::info!("Wifi DHCP info: {:?}", ip_info);

    Ok(())
}

pub fn init_wifi(
    driver: &mut EspWifi<'static>,
    sysloop: EspSystemEventLoop,
    settings: &WiFiSettings,
) -> Result<()> {
    match settings.mode {
        WiFiMode::Client => {
            create_wifi_ap(
                driver,
                sysloop,
                settings.ssid.as_str(),
                settings.password.as_str(),
            )?;
        }
        WiFiMode::Server => {
            connect_to_wifi_ap(
                driver,
                sysloop,
                settings.ssid.as_str(),
                settings.password.as_str(),
            )?;
        }
    }
    Ok(())
}
