mod bat;
mod bme680;
mod bsec2;
mod multicast;
mod rgb_led;
mod utils;
mod web;
mod wifi;

use esp_idf_hal::ledc::{config::TimerConfig, LedcDriver, LedcTimerDriver};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use log::info;
use std::env;
use std::time::Duration;

use crate::rgb_led::Color;
use crate::utils::LightSleep;
use crate::wifi::{Credential, WiFi};
use common::packet::{DeviceInfo, Header, Measurement, Packet, Payload};

// --------------------------------------------------------------------
// config
// --------------------------------------------------------------------
const ENV_STR: &str = include_str!("../.env");

const MES_SAMPLE_RATE: bsec2::SampleRate = bsec2::SampleRate::Ulp; // 5min (ulp), 3s (lp)
const MES_REPORT_INTERVAL_DIV: u32 = 3;

const ENABLE_WIFI: bool = true;
const ENABLE_LIGHT_SLEEP: bool = true;

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
// debug
//  cargo build && espflash /dev/ttyACM0 target/riscv32imc-esp-espidf/debug/firmware-m1s1 --monitor --speed 921600
// release
//  cargo build --release && espflash /dev/ttyACM0 target/riscv32imc-esp-espidf/release/firmware-m1s1 --monitor --speed 921600
//
// --------------------------------------------------------------------

#[allow(deprecated)]
fn main() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let part = esp_idf_svc::nvs::EspDefaultNvsPartition::take()?;
    let _nvs = esp_idf_svc::nvs::EspNvs::new(part, "default", true);

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

    let model: [u8; 16] = to_array(DEVICE_MODEL.as_bytes());

    bsec2::init()?;
    let bsec_version = bsec2::version()?;
    let mes_interval = MES_SAMPLE_RATE.sample_time_interval().unwrap();
    info!(
        "bsec2 v{}.{}.{}.{}",
        bsec_version[0], bsec_version[1], bsec_version[2], bsec_version[3]
    );
    {
        use bsec2::*;

        let sample_rate = MES_SAMPLE_RATE;

        let _sensor_inputs = bsec2::update_subscription(&[
            VirtualSensorConfiguration {
                sample_rate,
                sensor: VirtualSensor::HeatCompensatedTemperature,
            },
            VirtualSensorConfiguration {
                sample_rate,
                sensor: VirtualSensor::HeatCompensatedHumidity,
            },
            VirtualSensorConfiguration {
                sample_rate,
                sensor: VirtualSensor::Voc,
            },
            VirtualSensorConfiguration {
                sample_rate,
                sensor: VirtualSensor::StaticIAQ,
            },
            VirtualSensorConfiguration {
                sample_rate,
                sensor: VirtualSensor::RawGas,
            },
            VirtualSensorConfiguration {
                sample_rate,
                sensor: VirtualSensor::RawPressure,
            },
            VirtualSensorConfiguration {
                sample_rate,
                sensor: VirtualSensor::RawTemperature,
            },
            VirtualSensorConfiguration {
                sample_rate,
                sensor: VirtualSensor::StabilizationStatus,
            },
        ])?;
        // dbg!(sensor_inputs);
    }

    // peripherals
    let peripherals = Peripherals::take().unwrap();

    // configure wakeup pin and light sleep
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
    // parse credentials
    let wifi_ssids = wifi_ssid.split_terminator(';');
    let wifi_pws = wifi_pw.split_terminator(';');

    let credentials: Vec<_> = wifi_ssids
        .zip(wifi_pws)
        .map(|(ssid, pw)| Credential {
            ssid: ssid.into(),
            pw: pw.into(),
        })
        .collect();

    let sys_loop = EspSystemEventLoop::take()?;
    let mut wifi = WiFi::new(peripherals.modem, sys_loop, credentials)?;
    if ENABLE_WIFI {
        wifi.scan()?;
        wifi.start()?;
        wifi.connect_blocking()?;
        led.set_color(&Color::Green);
        std::thread::sleep(Duration::from_millis(500));
    }
    led.set_color(&Color::Black);

    std::thread::sleep(Duration::from_millis(3000));

    // multicast
    let mut mc_client = multicast::Client::new()?;

    // bme680
    bme680.setup(&bme680::Config::default())?;

    // adc
    let mut bat = bat::BatMonitor::new(peripherals.adc1, peripherals.pins.gpio1)?;

    let mut report_interval = 0;
    let mut _report_lock = None;
    let mut next_sample_instant = Duration::ZERO;
    let startup_time = utils::system_time();

    loop {
        // sample?
        if next_sample_instant.saturating_sub(utils::system_time()) == Duration::ZERO {
            log::info!("sample: {:?}", utils::system_time());
            let (outputs, next_call) = bsec2::sensor_control(utils::system_time(), &mut bme680)?;
            next_sample_instant = next_call;
            let (bat_cap, bat_v) = bat.capacity()?;

            log::info!(
                "current ts: {:?} s, next ts {:?} s",
                utils::system_time(),
                next_call
            );

            mc_client.enqueue(Packet {
                header: Header::new(device_id, utils::system_time().as_millis() as u64),
                payload: Payload::Measurement(Measurement {
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
                    header: Header::new(device_id, utils::system_time().as_millis() as u64),
                    payload: Payload::DeviceInfo(DeviceInfo {
                        uptime: (utils::system_time() - startup_time).as_secs(),
                        firmware_version: fw_version,
                        bsec_version,
                        model,
                        wifi_ssid: wifi.ssid().as_ref().map(|ssid| to_array(ssid.as_bytes())),
                        report_interval: mes_interval.as_secs() * MES_REPORT_INTERVAL_DIV as u64,
                        sample_interval: mes_interval.as_secs(),
                    }),
                })?;

                // prevent sleep until data has been transmitted
                if ENABLE_LIGHT_SLEEP {
                    println!(
                        "sleep request! {:?} - {:?}",
                        utils::system_time(),
                        next_call
                    );

                    _report_lock = Some(lightsleep.lock());
                }

                if ENABLE_WIFI {
                    wifi.start()?;
                    wifi.connect()?;
                }
            }

            if ENABLE_LIGHT_SLEEP {
                lightsleep.sleep_until(next_call);
            }
        }

        if wifi.is_connected() {
            log::info!("Broadcast pkgs");
            mc_client
                .broadcast_queue()
                .expect("Cannot transfer packets");

            if ENABLE_WIFI {
                wifi.stop().expect("Cannot stop wifi");
            }

            // drop the sleep lock, triggers sleep
            _report_lock = None;
        }

        std::thread::sleep(Duration::from_millis(10));
    }
}

fn to_array<const N: usize>(s: &[u8]) -> [u8; N] {
    assert!(s.len() <= N);

    let mut a = [0; N];
    a[0..s.len()].copy_from_slice(s);
    a
}
