use anyhow::Result;
use esp_idf_hal::{adc, gpio};

pub struct Bat {
    driver: adc::AdcDriver<'static, adc::ADC1>,
    channel: adc::AdcChannelDriver<'static, gpio::Gpio1, adc::Atten11dB<adc::ADC1>>,
}

impl Bat {
    pub fn new(adc1: esp_idf_hal::adc::ADC1, pin: gpio::Gpio1) -> Result<Self> {
        let channel: adc::AdcChannelDriver<_, adc::Atten11dB<adc::ADC1>> =
            adc::AdcChannelDriver::new(pin)?;
        let driver =
            esp_idf_hal::adc::AdcDriver::new(adc1, &adc::config::Config::new().calibration(true))?;

        Ok(Self { driver, channel })
    }

    pub fn read(&mut self) -> Result<u16> {
        let bat_adc = self.driver.read(&mut self.channel)?;
        Ok(bat_adc)
    }

    pub fn read_voltage(&mut self) -> Result<f32> {
        let bat_adc = self.read()?;

        // 12 bit adc, 2.5V @ 11dB
        let voltage = bat_adc as f32 * 4.17 / (0xFFF as f32) * 2.0;

        Ok(voltage)
    }
}
