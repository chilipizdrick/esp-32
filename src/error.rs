pub type Error = Box<dyn std::error::Error>;
pub type Result<T> = core::result::Result<T, Error>;

// #[derive(Debug, From)]
// pub enum Error {
//     Custom(String),
//
//     // Wifi
//     MissingSSIDError,
//     WiFiInitializationError(EspError),
//     // WiFiConfigurationError,
//     WiFiScanningError(EspError),
//     SSIDParseError(EspError),
//     PasswordParseError(EspError),
//     WiFiConnectionError(EspError),
// }
