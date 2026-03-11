#![allow(dead_code, unused)]
use anyhow::Result;
use esp_idf_hal::modem::WifiModemPeripheral;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::mqtt::client::*;
use esp_idf_svc::nvs::EspDefaultNvsPartition;
use esp_idf_svc::wifi::*;

pub fn send_mqtt_notification(modem: impl WifiModemPeripheral) -> Result<()> {
    let sys_loop = EspSystemEventLoop::take()?;
    let nvs = EspDefaultNvsPartition::take()?;

    // WiFi接続
    let mut wifi = BlockingWifi::wrap(EspWifi::new(modem, sys_loop.clone(), Some(nvs))?, sys_loop)?;

    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: "yxhhdjjdjfufhdjrffd".try_into().unwrap(),
        password: "tamasama_20302_kyawaii".try_into().unwrap(),
        ..Default::default()
    }))?;

    wifi.start()?;
    wifi.connect()?;
    wifi.wait_netif_up()?;
    println!("WiFi接続完了");

    // MQTT送信
    let (mut client, mut connection) = EspMqttClient::new(
        "mqtt://192.168.68.200:1883",
        &MqttClientConfiguration::default(),
    )?;

    // connectionを別スレッドで処理
    std::thread::spawn(move || while let Ok(msg) = connection.next() {});

    client.publish(
        "laundry/status",
        QoS::AtMostOnce,
        false,
        "finished".as_bytes(),
    )?;
    println!("MQTT送信完了");

    // 少し待ってから切断
    std::thread::sleep(std::time::Duration::from_millis(500));

    wifi.disconnect()?;
    wifi.stop()?;

    Ok(())
}
