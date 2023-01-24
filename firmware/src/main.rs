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

use esp_idf_hal::gpio::PinDriver;
use esp_idf_hal::ledc::{config::TimerConfig, LedcDriver, LedcTimerDriver};
use esp_idf_hal::peripherals::Peripherals;
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_svc::sntp::SyncStatus;
use esp_idf_sys;
use log::info;
use std::env;
use std::time::{Duration, Instant};

use crate::packet::Measurement;
use crate::rgb_led::Color;
use packet::{Header, Packet, Payload};

const ENV_STR: &str = include_str!("../.env");

const MES_INTERVAL: Duration = Duration::from_secs(30);

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

    // wu pin
    let wu_pin = PinDriver::input(peripherals.pins.gpio3)?;
    unsafe {
        // enable wakeup on gpio3
        esp_idf_sys::esp!(esp_idf_sys::gpio_wakeup_enable(
            wu_pin.pin(),
            esp_idf_sys::gpio_int_type_t_GPIO_INTR_HIGH_LEVEL,
        ))?;
        // enable timer wakeup
        esp_idf_sys::esp!(esp_idf_sys::esp_sleep_enable_timer_wakeup(
            MES_INTERVAL.as_micros() as u64
        ))?;
    }

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
        info!("BME680 found")
    }

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
    let sys_loop = EspSystemEventLoop::take()?;
    let mut wifi = wifi::setup_wifi(peripherals.modem, sys_loop, &wifi_ssid, &wifi_pw)?;
    led.set_color(&Color::Green);
    std::thread::sleep(Duration::from_millis(500));
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

    let mut last_mes = Instant::now() - MES_INTERVAL;

    loop {
        if Instant::now().duration_since(last_mes) >= MES_INTERVAL {
            last_mes = Instant::now();

            bme680.trigger_measurement()?;
            std::thread::sleep(Duration::from_millis(500)); // wait until data is ready

            if !bme680.is_busy()? {
                let mes = bme680.read_measurements()?;
                let v_bat = bat.read_voltage()?;
                // mc_client.send()?;

                wifi.connect()?;
                if wifi.is_connected()? {
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
                }

                //dbg!(mes);
            }
        } else {
            std::thread::sleep(Duration::from_millis(100));
        }

        // sleep if we are on battery
        if wu_pin.is_low() {
            info!("sleep...");
            wifi.stop()?;
            std::thread::sleep(Duration::from_millis(50));

            unsafe {
                esp_idf_sys::esp!(esp_idf_sys::esp_light_sleep_start())?;
            };

            std::thread::sleep(Duration::from_millis(50));
            info!("wakeup...");
            wifi.start()?;
        }
    }
}
