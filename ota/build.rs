// Necessary because of this issue: https://github.com/rust-lang/cargo/issues/9641
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // if !std::path::Path::new("cfg.toml").exists() {
    //     panic!("cfg.toml not found."); 
    // }
    // panic!("{:?}", std::env::var("CARGO_PKG_NAME"));
    embuild::build::CfgArgs::output_propagated("ESP_IDF")?;
    embuild::build::LinkArgs::output_propagated("ESP_IDF")?;
    Ok(())
}
