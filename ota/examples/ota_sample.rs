use anyhow::Result;
fn main() -> Result<()>{
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    // This is a very unrealistic example. You usually don't store the new app in the
    // old app. Instead you obtain it by downloading it from somewhere or similar.
    const NEW_APP: &[u8] = include_bytes!("../../asset/bootloader.bin");

    // Finds the next suitable OTA partition and erases it
    let mut ota = esp_ota::OtaUpdate::begin()?;

    // Write the app to flash. Normally you would download
    // the app and call `ota.write` every time you have obtained
    // a part of the app image. This example is not realistic,
    // since it has the entire new app bundled.
    for app_chunk in NEW_APP.chunks(4096) {
        ota.write(app_chunk)?;
    }

    // Performs validation of the newly written app image and completes the OTA update.
    let mut completed_ota = ota.finalize()?;

    // Sets the newly written to partition as the next partition to boot from.
    completed_ota.set_as_boot_partition()?;
    // Restarts the CPU, booting into the newly written app.
    completed_ota.restart();
}
