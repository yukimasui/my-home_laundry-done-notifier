use rumqttc::{Client, MqttOptions, QoS};
use std::process::Command;
use std::thread;

slint::include_modules!();

fn main() {
    let ui = MainWindow::new().unwrap();
    let ui_handle = ui.as_weak();

    let mut mqttoptions = MqttOptions::new("surface", "localhost", 1883);
    mqttoptions.set_keep_alive(std::time::Duration::from_secs(5));
    let (client, mut connection) = Client::new(mqttoptions, 10);
    client.subscribe("laundry/status", QoS::AtMostOnce).unwrap();

    thread::spawn(move || {
        for notification in connection.iter() {
            if let Ok(rumqttc::Event::Incoming(rumqttc::Packet::Publish(_))) = notification {
                let handle = ui_handle.clone();
                slint::invoke_from_event_loop(move || {
                    if let Some(ui) = handle.upgrade() {
                        ui.invoke_show_popup();
                    }
                })
                .unwrap();
            }
        }
    });

    ui.on_shutdown(|| {
        Command::new("sudo")
            .arg("shutdown")
            .arg("-h")
            .arg("now")
            .spawn()
            .unwrap();
    });
    ui.run().unwrap();
}
