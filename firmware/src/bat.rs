use anyhow::Result;
use esp_idf_hal::{
    adc::{self, config::Resolution},
    gpio,
};

pub struct BatMonitor {
    driver: adc::AdcDriver<'static, adc::ADC1>,
    channel: adc::AdcChannelDriver<'static, gpio::Gpio1, adc::Atten11dB<adc::ADC1>>,
}

impl BatMonitor {
    pub fn new(adc1: esp_idf_hal::adc::ADC1, pin: gpio::Gpio1) -> Result<Self> {
        let channel: adc::AdcChannelDriver<_, adc::Atten11dB<adc::ADC1>> =
            adc::AdcChannelDriver::new(pin)?;
        let driver = esp_idf_hal::adc::AdcDriver::new(
            adc1,
            &adc::config::Config::new()
                .calibration(true)
                .resolution(Resolution::Resolution12Bit),
        )?;

        Ok(Self { driver, channel })
    }

    pub fn read(&mut self) -> Result<u16> {
        let bat_adc = self.driver.read(&mut self.channel)?; // mV!

        Ok(bat_adc)
    }

    pub fn read_voltage(&mut self) -> Result<f32> {
        let bat_mv = self.read()?;

        // 12 bit adc, 2.45V @ 11dB
        let r1 = 10.0;
        let r2 = 10.0;
        let vout_over_vin = (r1 + r2) / r2;
        let voltage = bat_mv as f32 / 1000.0 * vout_over_vin;

        Ok(voltage)
    }

    pub fn capacity(&mut self) -> Result<(f32, f32)> {
        let voltage = self.read_voltage()?;

        // 4xAA battery: 1.5V=100%, 1.2V=0%
        let bat_count = 4.0;
        let v_min = 1.1 * bat_count;
        let v_max = 1.35 * bat_count;
        let v_range = v_max - v_min;

        let cap = ((voltage - v_min) / v_range).clamp(0.0, 1.0);

        Ok((cap * 100.0, voltage))
    }
}
