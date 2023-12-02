use std::thread::sleep;
use std::time::Duration;

fn main() {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let cnt = 10;

    let mut join_array = Vec::new();

    for i in 0..10 {
        let thread = std::thread::spawn(move || {
            sleep(Duration::from_millis(1000));
            println!("cnt: {}", i);
            println!("Hello from a thread!");
        });
        join_array.push(thread);
    }
    for i in 0..10 {
        let thread = std::thread::spawn(move || {
            println!("cnt: {}", i + 10);
            println!("Hello from a thread!");
        });
        join_array.push(thread);
    }
    for thread in join_array {
        thread.join().unwrap();
    }

    loop {}
}
