use anyhow::Result;

use embedded_svc::http::client::Client as HttpClient;
use embedded_svc::http::Method;
use embedded_svc::wifi::{AuthMethod, ClientConfiguration, Configuration};
use esp_idf_svc::http::client::EspHttpConnection;

use esp_idf_svc::wifi::{BlockingWifi, EspWifi};
use log::info;

pub struct Ota {}

impl Ota {
    pub fn new() -> Self {
        Ota {}
    }
    // execute the OTA
    pub fn connect_to_wifi(
        &self,
        wifi: &mut BlockingWifi<EspWifi<'static>>,
        wifi_ssid: &str,
        wifi_password: &str,
    ) -> Result<()> {
        let wifi_config = Configuration::Client(ClientConfiguration {
            ssid: wifi_ssid.into(),
            password: wifi_password.into(),
            auth_method: AuthMethod::WPA2Personal,
            bssid: None,
            channel: None,
        });

        info!("set wifi configuration...");

        wifi.set_configuration(&wifi_config)?;
        wifi.start()?;
        wifi.connect()?;

        info!("wifi connected!");

        wifi.wait_netif_up()?;
        info!("wifi netif up!");

        Ok(())
    }
    pub fn check_firmware_is_latest(&self, _url: &str, _filename: &str) -> Result<bool> {
        Ok(false)
    }
    pub fn download_firmware(&self, url: &str, filename: &str) -> Result<()> {
        info!("try to create http client...");
        let mut client = HttpClient::wrap(EspHttpConnection::new(&Default::default())?);

        info!("try to create request...");
        let header = [("accept", "binary/octet-stream"), ("connection", "close")];

        info!("request url: {}", url);
        let request = client.request(Method::Get, url, &header)?;

        info!("try to submit request...");
        let mut response = request.submit()?;
        info!("response received!");

        let status = response.status();
        info!("status: {}", status);

        // let (_, mut body): (_, &mut EspHttpConnection) = response.split();
        info!("==================================================");

        let mut ota = esp_ota::OtaUpdate::begin()?;

        info!("start to download firmware...");
        loop {
            let mut buf = [0; 1024];
            let byte_read = response.read(&mut buf).map_err(|e| e.0)?;
            if byte_read == 0 {
                break;
            }
            ota.write(&buf)?;
        }
        info!("download firmware success!");

        info!("try to finalize ota...");
        let mut completed_ota = ota.finalize()?;
        info!("finalize ota success!");

        info!("try to set as boot partition...");
        completed_ota.set_as_boot_partition()?;
        info!("set as boot partition success!");

        info!("try to restart...");
        completed_ota.restart();

        Ok(())
    }
    #[allow(dead_code)]
    fn print_buf(buf: &[u8], bytes_read: usize) {
        match std::str::from_utf8(&buf[0..bytes_read]) {
            Ok(string) => info!("body: {}", string),
            Err(e) => info!("body: {:?}", e),
        }
    }
}
