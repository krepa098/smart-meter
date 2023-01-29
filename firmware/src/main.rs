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

use esp_idf_hal::ledc::{config::TimerConfig, LedcDriver, LedcTimerDriver};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::sntp::SyncStatus;
use esp_idf_sys;
use log::info;
use std::env;
use std::time::Duration;

use crate::packet::Measurement;
use crate::rgb_led::Color;
use crate::utils::LightSleep;
use crate::wifi::{Credentials, WiFi};
use packet::{DeviceInfo, Header, Packet, Payload};

const ENV_STR: &str = include_str!("../.env");

// const MES_INTERVAL: Duration = Duration::from_secs(30);
const MES_INTERVAL: Duration = Duration::from_secs(3);
// const MES_INTERVAL: Duration = Duration::from_secs(50 * 60); // 5min (ulp)
// const MES_REPORT_INTERVAL_DIV: u32 = 3; // 15min
const MES_REPORT_INTERVAL_DIV: u32 = 20; // 1min
const ENABLE_WIFI: bool = true;

const DEVICE_MODEL: &str = "M1S1";

// --------------------------------------------------------------------
// pin definitions
// --------------------------------------------------------------------
// IO1:  VBUS_ADC
// IO3:  EXT_WU
// IO4:  SDA
// IO5:  SCL
// IO6:  LED_G
// IO7:  LED_R
// IO10: LED_B
// IO18: D-
// IO19: D+
// --------------------------------------------------------------------
//
// flash command:
// cargo build && espflash /dev/ttyACM0 target/riscv32imc-esp-espidf/debug/firmware-m1s1 --monitor --speed 921600
//
// --------------------------------------------------------------------

#[allow(deprecated)]
fn main() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let fw_version: [u8; 4] = [
        env!("CARGO_PKG_VERSION_MAJOR").parse().unwrap_or(0),
        env!("CARGO_PKG_VERSION_MINOR").parse().unwrap_or(0),
        env!("CARGO_PKG_VERSION_PATCH").parse().unwrap_or(0),
        env!("CARGO_PKG_VERSION_PRE").parse().unwrap_or(0),
    ];
    info!(
        "firmware v{}.{}.{}.{}",
        fw_version[0], fw_version[1], fw_version[2], fw_version[3]
    );
    // unique id from mac address
    let device_id = utils::device_id()?;
    info!("Device ID: {}", device_id);

    let mut model = [0; 16];
    model[0..DEVICE_MODEL.as_bytes().len()].copy_from_slice(DEVICE_MODEL.as_bytes());

    bsec2::init()?;
    let bsec_version = bsec2::version()?;
    info!(
        "bsec2 v{}.{}.{}.{}",
        bsec_version[0], bsec_version[1], bsec_version[2], bsec_version[3]
    );
    {
        use bsec2::*;

        let _sensor_inputs = bsec2::update_subscription(&[
            VirtualSensorConfiguration {
                sample_rate: SampleRate::Lp,
                sensor: VirtualSensor::HeatCompensatedTemperature,
            },
            VirtualSensorConfiguration {
                sample_rate: SampleRate::Lp,
                sensor: VirtualSensor::HeatCompensatedHumidity,
            },
            VirtualSensorConfiguration {
                sample_rate: SampleRate::Lp,
                sensor: VirtualSensor::Voc,
            },
            VirtualSensorConfiguration {
                sample_rate: SampleRate::Lp,
                sensor: VirtualSensor::StaticIAQ,
            },
            VirtualSensorConfiguration {
                sample_rate: SampleRate::Lp,
                sensor: VirtualSensor::RawGas,
            },
            VirtualSensorConfiguration {
                sample_rate: SampleRate::Lp,
                sensor: VirtualSensor::RawPressure,
            },
            VirtualSensorConfiguration {
                sample_rate: SampleRate::Lp,
                sensor: VirtualSensor::RawTemperature,
            },
            VirtualSensorConfiguration {
                sample_rate: SampleRate::Lp,
                sensor: VirtualSensor::StabilizationStatus,
            },
        ])?;
        // dbg!(sensor_inputs);
    }

    // peripherals
    let peripherals = Peripherals::take().unwrap();

    // configure wakeup pin for and light sleep
    let mut lightsleep = LightSleep::new(peripherals.pins.gpio3.into())?;

    // RGB led
    let mut led = {
        let ledc = peripherals.ledc;
        let config = TimerConfig::default();

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

    std::thread::sleep(Duration::from_millis(100));

    if bme680.chip_id_valid()? {
        info!("BME680 found");
    }
    bme680.reset()?;

    led.set_color(&Color::Green);
    std::thread::sleep(Duration::from_millis(100));
    led.set_color(&Color::Blue);
    std::thread::sleep(Duration::from_millis(100));
    led.set_color(&Color::Red);
    std::thread::sleep(Duration::from_millis(100));

    // wifi
    dotenvy::from_read(ENV_STR.as_bytes()).ok();
    let wifi_ssid = env::var("WIFI_SSID")?;
    let wifi_pw = env::var("WIFI_PW")?;
    let credentials = Credentials {
        ssid: wifi_ssid.as_str().into(),
        pw: wifi_pw.as_str().into(),
    };
    let sys_loop = EspSystemEventLoop::take()?;
    let mut wifi = WiFi::new(peripherals.modem, sys_loop)?;
    if ENABLE_WIFI {
        // wifi.scan()?;
        wifi.start(&credentials)?;
        wifi.connect_blocking()?;
        led.set_color(&Color::Green);
        std::thread::sleep(Duration::from_millis(500));
    }
    led.set_color(&Color::Black);

    // synchronize time (UTC)
    if wifi.is_connected()? {
        info!("Sync SNTP...");
        let sntp = esp_idf_svc::sntp::EspSntp::new_default()?;
        while sntp.get_sync_status() != SyncStatus::Completed {
            std::thread::sleep(Duration::from_millis(100));
        }
        wifi.stop()?;
    }

    std::thread::sleep(Duration::from_millis(3000));

    // multicast
    let mut mc_client = multicast::Client::new()?;

    // bme680
    bme680.setup(&bme680::Config::default())?;

    // adc
    let mut bat = bat::BatMonitor::new(peripherals.adc1, peripherals.pins.gpio1)?;

    let mut report_interval = 0;
    let mut report_lock = None;
    let mut next_sample_instant = None;
    let startup_time = utils::system_time();

    loop {
        if let Some(inst) = next_sample_instant {
            utils::thread_sleep_until(inst);
        }
        println!("sample: {:?}", utils::system_time());
        let (outputs, next_call) = bsec2::sensor_control(utils::system_time(), &mut bme680)?;
        next_sample_instant = Some(next_call);
        let (bat_cap, bat_v) = bat.capacity()?;

        println!("current: {:?}, next {:?}", utils::system_time(), next_call);

        mc_client.enqueue(Packet {
            header: Header::with_device_id(device_id),
            payload: Payload::Measurement(Measurement {
                timestamp: utils::system_time().as_millis() as u64,
                temperature: outputs.heat_compensated_temperature.map(|f| f.signal),
                pressure: outputs.raw_pressure.map(|f| f.signal),
                humidity: outputs.heat_compensated_humidity.map(|f| f.signal),
                air_quality: outputs.static_iaq.map(|f| f.signal),
                bat_voltage: Some(bat_v),
                bat_capacity: Some(bat_cap),
            }),
        })?;

        // send?
        report_interval += 1;
        if report_interval == MES_REPORT_INTERVAL_DIV {
            report_interval = 0;

            mc_client.enqueue(Packet {
                header: Header::with_device_id(device_id),
                payload: Payload::DeviceInfo(DeviceInfo {
                    uptime: (utils::system_time() - startup_time).as_millis() as u64,
                    firmware_version: fw_version,
                    bsec_version,
                    model,
                }),
            })?;

            println!("lock!");
            report_lock = Some(lightsleep.lock());

            if ENABLE_WIFI {
                wifi.start(&credentials)?;
                wifi.connect()?;
            }
        }

        if wifi.is_connected()? && report_lock.is_some() {
            println!("broadcast!");
            mc_client.broadcast_queue()?;
            report_lock = None;

            if ENABLE_WIFI {
                wifi.stop()?;
            }
        }

        println!(
            "sleep request! {:?} - {:?}",
            utils::system_time(),
            next_call
        );

        lightsleep.sleep_until(next_call - Duration::from_millis(10));
    }
}
