use std::time::Duration;

use anyhow::{bail, Result};
use esp_idf_hal::gpio::{AnyIOPin, Input, PinDriver};
use esp_idf_sys::{esp, esp_efuse_mac_get_default};

pub fn mac_addr() -> Result<([u8; 6])> {
    let mut mac = [0_u8; 6];
    let ret = unsafe { esp_efuse_mac_get_default(mac.as_mut_ptr()) };
    if ret != 0 {
        bail!("Cannot get MAC ADDR");
    }

    Ok(mac)
}

pub fn device_id() -> Result<u32> {
    let mac = mac_addr()?;
    let v1 = mac[0] as u32 | (mac[1] as u32) << 8 | (mac[2] as u32) << 16 | (mac[3] as u32) << 24;
    let v2 = mac[4] as u32 | (mac[5] as u32) << 8;
    Ok(v1 ^ v2)
}

pub fn system_time() -> Duration {
    esp_idf_svc::systime::EspSystemTime {}.now()
}

struct SleepShared {
    counter: u8,
    next_wakeup: Option<Duration>,
}

impl SleepShared {
    fn do_sleep(&self) {
        if let Some(next_wakeup) = self.next_wakeup {
            if self.counter == 0 {
                if let Some(dur) = next_wakeup.checked_sub(system_time()) {
                    // sleep for dur
                    unsafe {
                        esp!(esp_idf_sys::esp_sleep_enable_timer_wakeup(
                            dur.as_micros() as u64
                        ))
                        .unwrap();
                        esp_idf_sys::esp_light_sleep_start();
                    };
                }
            }
        }
    }
}

pub fn thread_sleep_until(instant: Duration) {
    if let Some(dur) = instant.checked_sub(system_time()) {
        println!("Sleep for {:?}", dur);
        std::thread::sleep(dur);
    }
}

pub struct LightSleep {
    shared: std::sync::Arc<std::sync::Mutex<SleepShared>>,
    _pin_driver: PinDriver<'static, AnyIOPin, Input>,
}

impl LightSleep {
    pub fn new(pin: AnyIOPin) -> Result<Self> {
        // enable wakeup on pin
        let pin_driver = PinDriver::input(pin)?;
        unsafe {
            esp_idf_sys::esp!(esp_idf_sys::gpio_wakeup_enable(
                pin_driver.pin(),
                esp_idf_sys::gpio_int_type_t_GPIO_INTR_HIGH_LEVEL,
            ))?;
        }

        let shared = std::sync::Arc::new(std::sync::Mutex::new(SleepShared {
            counter: 0,
            next_wakeup: None,
        }));

        Ok(Self {
            shared,
            _pin_driver: pin_driver,
        })
    }

    pub fn sleep_until(&mut self, instant: Duration) {
        if let Ok(mut shared) = self.shared.lock() {
            shared.next_wakeup = Some(instant);
            shared.do_sleep();
        }
    }

    pub fn lock(&self) -> SleepInhibitor {
        if let Ok(mut shared) = self.shared.lock() {
            shared.counter += 1;
        }

        SleepInhibitor {
            shared: self.shared.clone(),
        }
    }
}

pub struct SleepInhibitor {
    shared: std::sync::Arc<std::sync::Mutex<SleepShared>>,
}

impl Drop for SleepInhibitor {
    fn drop(&mut self) {
        if let Ok(mut shared) = self.shared.lock() {
            shared.counter -= 1;
            shared.do_sleep();
        }
    }
}
