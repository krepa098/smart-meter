mod bat;
mod bme680;
mod bsec2;
mod mqtt;
mod multicast;
mod packet;
mod rgb_led;
mod utils;
mod web;
mod wifi;
mod zero;

use esp_idf_hal::peripherals::Peripherals;
use esp_idf_hal::{
    ledc::{config::TimerConfig, LedcDriver, LedcTimerDriver},
    prelude::*,
};
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::sntp::SyncStatus;
use esp_idf_sys;
use log::info;
use std::env;
use std::time::Duration;

use crate::packet::Measurement;
use crate::rgb_led::Color;
use packet::{Header, Packet, Payload};

const ENV_STR: &str = include_str!("../.env");

// -----------------
// pin definitions
// -----------------
// IO1:  VBUS_ADC
// IO3:  EXT_WU
// IO4:  SDA
// IO5:  SCL
// IO6:  LED_G
// IO7:  LED_R
// IO10: LED_B
// IO18: D-
// IO19: D+
//
// flash command:
// cargo build && espflash /dev/ttyACM0 target/riscv32imc-esp-espidf/debug/firmware-m1s1 --monitor --speed 921600

#[allow(deprecated)]
fn main() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    // peripherals
    let peripherals = Peripherals::take().unwrap();

    // RGB led
    let mut led = {
        let ledc = peripherals.ledc;
        let config = TimerConfig::default().frequency(5.kHz().into());

        let timer_r = LedcTimerDriver::new(ledc.timer0, &config)?;
        let timer_g = LedcTimerDriver::new(ledc.timer1, &config)?;
        let timer_b = LedcTimerDriver::new(ledc.timer2, &config)?;

        let ch_r = LedcDriver::new(ledc.channel0, timer_r, peripherals.pins.gpio7)?;
        let ch_g = LedcDriver::new(ledc.channel1, timer_g, peripherals.pins.gpio6)?;
        let ch_b = LedcDriver::new(ledc.channel2, timer_b, peripherals.pins.gpio10)?;

        rgb_led::RGBLedPwm {
            r: ch_r,
            g: ch_g,
            b: ch_b,
            rgb: (0, 0, 0),
        }
    };

    led.set_color(&Color::Red);

    let mut bme680 = {
        let i2c = peripherals.i2c0;
        let sda = peripherals.pins.gpio4;
        let scl = peripherals.pins.gpio5;

        bme680::Device::new(i2c, sda, scl)
    }?;

    std::thread::sleep(std::time::Duration::from_millis(100));

    if bme680.chip_id_valid()? {
        info!("BME680 found")
    }

    led.set_color(&Color::Green);
    std::thread::sleep(std::time::Duration::from_millis(100));
    led.set_color(&Color::Blue);
    std::thread::sleep(std::time::Duration::from_millis(100));
    led.set_color(&Color::Red);
    std::thread::sleep(std::time::Duration::from_millis(100));

    // wifi
    dotenvy::from_read(ENV_STR.as_bytes()).ok();
    let wifi_ssid = env::var("WIFI_SSID")?;
    let wifi_pw = env::var("WIFI_PW")?;
    let sys_loop = EspSystemEventLoop::take()?;
    let _wifi = wifi::setup_wifi(peripherals.modem, sys_loop, &wifi_ssid, &wifi_pw)?;
    led.set_color(&Color::Green);
    std::thread::sleep(std::time::Duration::from_millis(500));
    led.set_color(&Color::Black);

    let device_id = utils::device_id()?;

    // synchronize time
    info!("Sync SNTP...");
    let sntp = esp_idf_svc::sntp::EspSntp::new_default()?;

    while sntp.get_sync_status() != SyncStatus::Completed {
        std::thread::sleep(Duration::from_millis(100));
    }

    // multicast
    let mut mc_client = multicast::Client::new()?;

    // bme680
    bme680.setup(&bme680::Config::default())?;
    bme680.trigger_measurement()?;

    // adc
    let mut bat = bat::Bat::new(peripherals.adc1, peripherals.pins.gpio1)?;
    // let a1 = esp_idf_hal::adc::AdcChannelDriver::new::<_, adc::Atten11dB<adc::ADC1>>(
    //     peripherals.pins.gpio1,
    // )?;
    // let adc_driver = esp_idf_hal::adc::AdcDriver::new(
    //     peripherals.adc1,
    //     &adc::config::Config::new().calibration(true),
    // )?;
    // let v_bat = adc_driver.read(&mut a1)?;

    loop {
        std::thread::sleep(std::time::Duration::from_millis(5000));
        // std::thread::sleep(std::time::Duration::from_secs(10 * 60));

        if !bme680.is_busy()? {
            let mes = bme680.read_measurements()?;
            let v_bat = bat.read_voltage()?;
            // mc_client.send()?;
            mc_client.broadcast(&Packet {
                header: Header::with_device_id(device_id),
                payload: Payload::Measurements(Measurement {
                    timestamp: mes.timestamp,
                    temperature: mes.temperature,
                    pressure: mes.pressure,
                    humidity: mes.humidity,
                    air_quality: mes.air_quality,
                    v_bat: Some(v_bat),
                }),
            })?;
            //dbg!(mes);

            bme680.trigger_measurement()?;
        }
    }
}
