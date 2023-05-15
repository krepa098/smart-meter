use anyhow::Result;
use arbitrary_int::*;
use bitbybit::bitfield;
use esp_idf_hal::gpio::{InputPin, OutputPin};
use esp_idf_hal::units::*;
use esp_idf_hal::{i2c, peripheral};
use log::info;
use num_enum::TryFromPrimitive;

const DEVICE_ADDR: u8 = 0x76;
const CHIP_ID: u8 = 0x61;
const TIMEOUT: u32 = 10000;
const CONST_ARRAY1: [f32; 16] = [
    1.0, 1.0, 1.0, 1.0, 1.0, 0.99, 1.0, 0.992, 1.0, 1.0, 0.998, 0.995, 1.0, 0.99, 1.0, 1.0,
];
const CONST_ARRAY2: [f32; 16] = [
    8000000.0,
    4000000.0,
    2000000.0,
    1000000.0,
    499500.4995,
    248262.1648,
    125000.0,
    63004.03226,
    31281.28128,
    15625.0,
    7812.5,
    3906.25,
    1953.125,
    976.5625,
    488.28125,
    244.140625,
];

#[allow(unused)]
mod register {
    pub const STATUS: u8 = 0x73;
    pub const RESET: u8 = 0xE0;
    pub const ID: u8 = 0xD0;
    pub const CONFIG: u8 = 0x75;
    pub const CTRL_MEAS: u8 = 0x74; // |osrs_t<2:0>|osrs_p<2:0>|mode<1:0>|
    pub const CTRL_HUM: u8 = 0x72;
    pub const CTRL_GAS_1: u8 = 0x71;
    pub const CTRL_GAS_0: u8 = 0x70;
    pub const GAS_WAIT_0: u8 = 0x64; // 0x64...0x6D
    pub const RES_HEAT_0: u8 = 0x63; // 0x5A...0x63
    pub const IDAC_HEAT_0: u8 = 0x59; // 0x50...0x59
    pub const GAS_R_LSB: u8 = 0x2B;
    pub const GAS_R_MSB: u8 = 0x2A;
    pub const HUM_LSB: u8 = 0x26;
    pub const HUM_MSB: u8 = 0x25;
    pub const TEMP_XLSB: u8 = 0x24;
    pub const TEMP_LSB: u8 = 0x23;
    pub const TEMP_MSB: u8 = 0x22;
    pub const PRESS_XLSB: u8 = 0x21;
    pub const PRESS_LSB: u8 = 0x20;
    pub const PRESS_MSB: u8 = 0x1F;
    pub const MEAS_STATUS_0: u8 = 0x1D;
}

#[bitfield(u8)]
struct Ctrl_meas {
    #[bits(5..=7, rw)]
    osrs_t: u3,

    #[bits(2..=4, rw)]
    osrs_p: u3,

    #[bits(0..=1, rw)]
    mode: u2,
}

#[bitfield(u8)]
struct Ctrl_hum {
    #[bits(0..=2, rw)]
    osrs_h: u3,
}

#[bitfield(u8)]
struct Ctrl_gas_1 {
    #[bits(0..=3, rw)]
    nb_conv: u4,

    #[bit(4, rw)]
    run_gas: bool,
}

#[bitfield(u8)]
struct Ctrl_gas_0 {
    #[bit(3, rw)]
    heat_off: bool,
}

#[bitfield(u8)]
struct Res_heat_range {
    #[bits(4..=5, r)]
    range: u2,
}

#[bitfield(u8)]
struct Gas_r_lsb {
    #[bits(6..=7, r)]
    gas_r: u2,

    #[bit(5, r)]
    gas_valid_r: bool,

    #[bits(0..=3, r)]
    gas_range_r: u4,
}

#[bitfield(u8)]
struct Meas_status_0 {
    #[bit(7, r)]
    new_data_0: bool,

    #[bit(6, r)]
    gas_measuring: bool,

    #[bit(5, r)]
    measuring: bool,

    #[bits(0..=3, r)]
    gas_meas_index: u4,
}

#[bitfield(u8)]
struct Gas_wait_x {
    #[bits(0..=5, rw)]
    timer: u6,

    #[bits(6..=7, rw)]
    div: u2,
}

pub struct Device {
    interface: i2c::I2cDriver<'static>,
    cal: Calibration,
    t_offset: f32,
    p_offset: f32,
    h_offset: f32,
}

#[derive(Default, Debug)]
#[allow(unused)]
pub struct Calibration {
    // temperature sensor
    par_t1: u16, // 0xE9/0xEA
    par_t2: i16, // 0x8A/0x8B
    par_t3: i8,  // 0x8C

    // pressure sensor
    par_p1: u16, // 0x8E/0x8F
    par_p2: i16, // 0x90/0x91
    par_p3: i8,  // 0x92
    par_p4: i16, // 0x94/0x95
    par_p5: i16, // 0x96/0x97
    par_p6: i8,  // 0x99
    par_p7: i8,  // 0x98
    par_p8: i16, // 0x9C/0x9D
    par_p9: i16, // 0x9E/0x9F
    par_p10: u8, // 0xA0

    // humidity
    par_h1: u16, // 0xE2<7:4>/0xE3
    par_h2: u16, // 0xE2<7:4>/0xE1
    par_h3: i8,  // 0xE4,
    par_h4: i8,  // 0xE5,
    par_h5: i8,  // 0xE6,
    par_h6: u8,  // 0xE7,
    par_h7: i8,  // 0xE8,

    // gas sensor
    par_g1: i8,         // 0xED
    par_g2: i16,        // 0xEB/0xEC
    par_g3: i8,         // 0xEE
    res_heat_range: u8, // 0x02 <5:4>
    res_heat_val: i8,   // 0x00
    range_sw_err: i8,   // 0x04 <7:4>
}

#[repr(u8)]
#[allow(unused)]
#[derive(Debug, Clone, Copy, TryFromPrimitive)]
pub enum Oversampling {
    Skip = 0,
    X1 = 0b001,
    X2 = 0b010,
    X4 = 0b011,
    X8 = 0b100,
    X16 = 0b101,
}

#[repr(u8)]
#[allow(unused)]
#[derive(Debug, Clone, Copy)]
pub enum IIRFilterSize {
    Fc0 = 0b000,
    Fc1 = 0b001,
    Fc3 = 0b010,
    Fc7 = 0b011,
    Fc15 = 0b100,
    Fc31 = 0b101,
    Fc63 = 0b110,
    Fc127 = 0b111,
}

#[repr(u8)]
#[allow(unused)]
#[derive(Debug, Clone, Copy)]
enum Mode {
    Sleep = 0b00,
    Forced = 0b01,
}

#[derive(Debug, Default, serde::Serialize)]
#[allow(unused)]
pub struct Measurements {
    pub timestamp: u64,
    pub temperature: Option<f32>, // °C
    pub pressure: Option<f32>,    // hPa
    pub humidity: Option<f32>,    // percent
    pub gas_res: Option<f32>,     // ohm
}

#[derive(Debug)]
#[allow(unused)]
pub struct Config {
    pub t_oversampling: Oversampling,
    pub p_oversampling: Oversampling,
    pub h_oversampling: Oversampling,
    pub iir_filter: IIRFilterSize,
    pub gas_heater_temp: f32,
    pub gas_heater_duration_ms: u16,
    pub gas_enabled: bool,
    pub gas_profile: u8, // 0-9
}

impl Default for Config {
    fn default() -> Self {
        Self {
            t_oversampling: Oversampling::X2,
            p_oversampling: Oversampling::X4,
            h_oversampling: Oversampling::X2,
            iir_filter: IIRFilterSize::Fc0,
            gas_heater_temp: 300.0,
            gas_heater_duration_ms: 100,
            gas_enabled: true,
            gas_profile: 0,
        }
    }
}

#[allow(unused)]
impl Device {
    pub fn new(
        i2c: impl peripheral::Peripheral<P = impl i2c::I2c> + 'static,
        sda: impl peripheral::Peripheral<P = impl InputPin + OutputPin> + 'static,
        scl: impl peripheral::Peripheral<P = impl InputPin + OutputPin> + 'static,
    ) -> Result<Self> {
        let i2c_conf = i2c::I2cConfig::new()
            .baudrate(400.kHz().into())
            .scl_enable_pullup(false)
            .sda_enable_pullup(false);
        let interface = i2c::I2cDriver::new(i2c, sda, scl, &i2c_conf)?;

        let mut device = Self {
            interface,
            cal: Default::default(),
            t_offset: -0.49,
            p_offset: 30.0,
            h_offset: 6.0,
        };

        // read calibration / factory data
        // coefficients start at COEFF_ADDR (0x89)
        device.cal = Calibration {
            // temperature sensor
            par_t1: device.read_reg16_lsb_msb(0xE9)?,
            par_t2: device.read_reg16_lsb_msb(0x8A)? as i16,
            par_t3: device.read_reg(0x8C)? as i8,

            // pressure sensor
            par_p1: device.read_reg16_lsb_msb(0x8E)?,
            par_p2: device.read_reg16_lsb_msb(0x90)? as i16,
            par_p3: device.read_reg(0x92)? as i8,
            par_p4: device.read_reg16_lsb_msb(0x94)? as i16,
            par_p5: device.read_reg16_lsb_msb(0x96)? as i16,
            par_p6: device.read_reg(0x99)? as i8,
            par_p7: device.read_reg(0x98)? as i8,
            par_p8: device.read_reg16_lsb_msb(0x9C)? as i16,
            par_p9: device.read_reg16_lsb_msb(0x9E)? as i16,
            par_p10: device.read_reg(0xA0)?,

            // humidity
            par_h1: device.read_reg16_lsb_msb(0xE2)? >> 4, // 12bit (0xE3|0xE2<7:4>)
            par_h2: device.read_reg16_msb_lsb(0xE1)? >> 4, // 12bit
            par_h3: device.read_reg(0xE4)? as i8,
            par_h4: device.read_reg(0xE5)? as i8,
            par_h5: device.read_reg(0xE6)? as i8,
            par_h6: device.read_reg(0xE7)?,
            par_h7: device.read_reg(0xE8)? as i8,

            // gas sensor
            par_g1: device.read_reg(0xED)? as i8,
            par_g2: device.read_reg16_lsb_msb(0xEB)? as i16,
            par_g3: device.read_reg(0xEE)? as i8,
            res_heat_range: Res_heat_range::new_with_raw_value(device.read_reg(0x02)?)
                .range()
                .into(),
            res_heat_val: device.read_reg(0x00)? as i8,
            range_sw_err: device.read_reg(0x04)? as i8 >> 4,
        };

        info!("Calibration data:\n{:?}", &device.cal);

        Ok(device)
    }

    pub fn setup(&mut self, config: &Config) -> Result<()> {
        // Setup oversampling
        let ctrl_meas = Ctrl_meas::new_with_raw_value(self.read_reg(register::CTRL_MEAS)?)
            .with_osrs_p(u3::new(config.p_oversampling as u8))
            .with_osrs_t(u3::new(config.t_oversampling as u8))
            .with_mode(u2::new(Mode::Sleep as u8));

        let ctrl_hum = Ctrl_hum::new_with_raw_value(self.read_reg(register::CTRL_HUM)?)
            .with_osrs_h(u3::new(config.h_oversampling as u8));

        let buf = [
            register::CTRL_HUM,
            ctrl_hum.raw_value,
            register::CTRL_MEAS,
            ctrl_meas.raw_value,
        ];
        self.interface.write(DEVICE_ADDR, &buf, TIMEOUT)?;

        // set gas sensor hotplate temperature
        self.write_reg(
            register::GAS_WAIT_0 + config.gas_profile,
            calc_gas_wait_time(config.gas_heater_duration_ms),
        )?;
        self.write_reg(
            register::RES_HEAT_0 + config.gas_profile,
            calc_glas_res_heat(config.gas_heater_temp, 20.0, &self.cal),
        )?;

        // Ctrl_gas_1 nb_conv<3:0> to 0x0
        // Set run_gas_l to 1 to enable gas measurements
        let ctrl_gas_1 = Ctrl_gas_1::new_with_raw_value(self.read_reg(register::CTRL_GAS_1)?)
            .with_nb_conv(u4::new(config.gas_profile)) // index of heater set-point
            .with_run_gas(config.gas_enabled); // enable gas conversion
        self.write_reg(register::CTRL_GAS_1, ctrl_gas_1.raw_value)?;

        Ok(())
    }

    fn setup_gas(&mut self, profile: u8, target_temperature: f32, heating_duration: f32) {
        // notice: about 20-30 ms are needed for the heater to reach target temperature
        assert!(profile < 10);
    }

    pub fn trigger_measurement(&mut self) -> Result<()> {
        let ctrl_meas = Ctrl_meas::new_with_raw_value(self.read_reg(register::CTRL_MEAS)?)
            .with_mode(u2::new(Mode::Forced as u8));
        self.write_reg(register::CTRL_MEAS, ctrl_meas.raw_value)?;
        Ok(())
    }

    pub fn reset(&mut self) -> Result<()> {
        self.interface
            .write(DEVICE_ADDR, &[register::RESET as u8, 0xB6], TIMEOUT)?;
        Ok(())
    }

    pub fn chip_id(&mut self) -> Result<u8> {
        self.read_reg(register::ID)
    }

    pub fn chip_id_valid(&mut self) -> Result<bool> {
        Ok(self.chip_id()? == CHIP_ID)
    }

    fn read_reg(&mut self, reg: u8) -> Result<u8> {
        let mut buffer = [0_u8; 1];
        self.interface
            .write_read(DEVICE_ADDR, &[reg as u8], &mut buffer, TIMEOUT)?;

        Ok(buffer[0])
    }

    // fn read_reg_<T: Clone + Into<i8>>(&mut self, reg: u8) -> Result<T> {
    //     let mut buffer = [0_u8; 1];
    //     self.interface
    //         .write_read(DEVICE_ADDR, &[reg as u8], &mut buffer, TIMEOUT)?;

    //     Ok(buffer[0].into())
    // }

    fn read_reg16_msb_lsb(&mut self, reg: u8) -> Result<u16> {
        // bme680 autoincrements the register
        // msb comes first
        let mut buffer = [0_u8; 2];
        self.interface
            .write_read(DEVICE_ADDR, &[reg as u8], &mut buffer, TIMEOUT)?;

        Ok((buffer[0] as u16) << 8 | buffer[1] as u16)
    }

    fn read_reg16_lsb_msb(&mut self, reg: u8) -> Result<u16> {
        // bme680 autoincrements the register
        // lsb comes first
        let mut buffer = [0_u8; 2];
        self.interface
            .write_read(DEVICE_ADDR, &[reg as u8], &mut buffer, TIMEOUT)?;

        Ok((buffer[1] as u16) << 8 | buffer[0] as u16)
    }

    fn write_reg(&mut self, reg: u8, data: u8) -> Result<()> {
        self.interface.write(DEVICE_ADDR, &[reg, data], TIMEOUT)?;
        Ok(())
    }

    fn read_temp_adc(&mut self) -> Result<u32> {
        Ok(((self.read_reg16_msb_lsb(register::TEMP_MSB)? as u32) << 4)
            | ((self.read_reg(register::TEMP_XLSB)? as u32) >> 4))
    }

    fn read_press_adc(&mut self) -> Result<u32> {
        Ok(
            ((self.read_reg16_msb_lsb(register::PRESS_MSB)? as u32) << 4)
                | ((self.read_reg(register::PRESS_XLSB)? as u32) >> 4),
        )
    }

    fn read_hum_adc(&mut self) -> Result<u32> {
        Ok(self.read_reg16_msb_lsb(register::HUM_MSB)? as u32)
    }

    fn read_gas_adc(&mut self) -> Result<u32> {
        Ok(((self.read_reg(register::GAS_R_MSB)? as u32) << 2
            | (self.read_reg(register::GAS_R_LSB)? as u32) >> 6))
    }

    pub fn read_measurements(&mut self) -> Result<Measurements> {
        // temperature
        let (temp_comp, t_fine) = {
            // let t_fine_offset = if self.t_offset > 0.0 {
            //     (((((self.t_offset.abs() as i32) * 100) << 8) - 128) / 5)
            // } else {
            //     -(((((self.t_offset.abs() as i32) * 100) << 8) - 128) / 5)
            // };
            let t_fine_offset = self.t_offset * 5120.0;
            // 20 bit value
            // TEMP_MSB<7:0>|TEMP_LSB<7:0>|TEMP_XLSB<7:4>
            let temp_adc = self.read_temp_adc()? as f32;
            let par_t1 = self.cal.par_t1 as f32;
            let par_t2 = self.cal.par_t2 as f32;
            let par_t3 = self.cal.par_t3 as f32;
            let var1 = ((temp_adc / 16384.0) - (par_t1 / 1024.0)) * par_t2;
            let var2 = (((temp_adc / 131072.0) - (par_t1 / 8192.0))
                * ((temp_adc / 131072.0) - (par_t1 / 8192.0)))
                * (par_t3 * 16.0);
            let t_fine = var1 + var2 + t_fine_offset;

            // in °C
            (t_fine / 5120.0, t_fine)
        };

        // pressure
        let press_comp = {
            // 20 bit value
            // PRESS_MSB<7:0>|PRESS_LSB<7:0>|PRESS_XLSB<7:4>
            let press_adc = self.read_press_adc()? as f32;
            let par_p1 = self.cal.par_p1 as f32;
            let par_p2 = self.cal.par_p2 as f32;
            let par_p3 = self.cal.par_p3 as f32;
            let par_p4 = self.cal.par_p4 as f32;
            let par_p5 = self.cal.par_p5 as f32;
            let par_p6 = self.cal.par_p6 as f32;
            let par_p7 = self.cal.par_p7 as f32;
            let par_p8 = self.cal.par_p8 as f32;
            let par_p9 = self.cal.par_p9 as f32;
            let par_p10 = self.cal.par_p10 as f32;

            let var1 = (t_fine / 2.0) - 64000.0;
            let var2 = var1 * var1 * (par_p6 / 131072.0);
            let var2 = var2 + (var1 * par_p5 * 2.0);
            let var2 = (var2 / 4.0) + (par_p4 * 65536.0);
            let var1 = (((par_p3 * var1 * var1) / 16384.0) + (par_p2 * var1)) / 524288.0;
            let var1 = (1.0 + (var1 / 32768.0)) * par_p1;
            let press_comp = 1048576.0 - press_adc;
            let press_comp = ((press_comp - (var2 / 4096.0)) * 6250.0) / var1;
            let var1 = (par_p9 * press_comp * press_comp) / 2147483648.0;
            let var2 = press_comp * (par_p8 / 32768.0);
            let var3 = (press_comp / 256.0)
                * (press_comp / 256.0)
                * (press_comp / 256.0)
                * (par_p10 / 131072.0);

            // in hPa
            (press_comp + (var1 + var2 + var3 + (par_p7 * 128.0)) / 16.0)
        };

        // humidity
        let hum_comp = {
            let hum_adc = self.read_hum_adc()? as f32;
            let par_h1 = self.cal.par_h1 as f32;
            let par_h2 = self.cal.par_h2 as f32;
            let par_h3 = self.cal.par_h3 as f32;
            let par_h4 = self.cal.par_h4 as f32;
            let par_h5 = self.cal.par_h5 as f32;
            let par_h6 = self.cal.par_h6 as f32;
            let par_h7 = self.cal.par_h7 as f32;

            let var1 = hum_adc - ((par_h1 * 16.0) + ((par_h3 / 2.0) * temp_comp));
            let var2 = var1
                * ((par_h2 / 262144.0)
                    * (1.0
                        + ((par_h4 / 16384.0) * temp_comp)
                        + ((par_h5 / 1048576.0) * temp_comp * temp_comp)));
            let var3 = par_h6 / 16384.0;
            let var4 = par_h7 / 2097152.0;

            // in percent
            (var2 + ((var3 + (var4 * temp_comp)) * var2 * var2)).clamp(0.0, 100.0)
        };

        // gas
        let gas_r_lsb = Gas_r_lsb::new_with_raw_value(self.read_reg(register::GAS_R_LSB)?);
        let gas_res = if gas_r_lsb.gas_valid_r() {
            let gas_adc = self.read_gas_adc()? as f32;
            let gas_range: u32 = gas_r_lsb.gas_range_r().into();
            let range_sw_err = self.cal.range_sw_err as f32;

            let var1 = (1340.0 + 5.0 * range_sw_err) * CONST_ARRAY1[gas_range as usize];

            // in ohm
            Some(var1 * CONST_ARRAY2[gas_range as usize] / (gas_adc as f32 - 512.0 + var1))
        } else {
            None
        };

        //unsafe { bsec2::bsec_init() };

        let timestamp = std::time::SystemTime::now().elapsed()?;

        Ok(Measurements {
            timestamp: timestamp.as_secs(),
            temperature: Some(temp_comp),
            pressure: Some(press_comp + self.p_offset),
            humidity: Some(hum_comp + self.h_offset),
            gas_res,
        })
    }

    pub fn is_busy(&mut self) -> Result<bool> {
        let meas_status_0 =
            Meas_status_0::new_with_raw_value(self.read_reg(register::MEAS_STATUS_0)?);

        Ok(meas_status_0.measuring())
    }
}

pub fn calc_gas_wait_time(time: u16) -> u8 {
    // 6 bit (64ms) time steps, 1ms each
    // 2 bit multiplication factor (1, 4, 16, 64)
    let div_coeffs = [1, 4, 16, 64];
    let mut div_reg = 0;
    let mut time_reg = 0;
    for (i, d) in div_coeffs.iter().enumerate() {
        if time / d <= 64 {
            div_reg = i as u8;
            time_reg = ((time / d) + 1) as u8;
            break;
        }
    }

    Gas_wait_x::new_with_raw_value(0)
        .with_div(u2::new(div_reg))
        .with_timer(u6::new(time_reg))
        .raw_value()
}

fn calc_glas_res_heat(target_temp: f32, amb_temp: f32, cal: &Calibration) -> u8 {
    let par_g1 = cal.par_g1 as f32;
    let par_g2 = cal.par_g2 as f32;
    let par_g3 = cal.par_g3 as f32;
    let res_heat_val = cal.res_heat_val as f32;
    let res_heat_range = cal.res_heat_range as f32;

    let var1 = par_g1 / 16.0 + 49.0;
    let var2 = (par_g2 / 32768.0 * 0.0005) + 0.00235;
    let var3 = par_g3 / 1024.0;
    let var4 = var1 * (1.0 + (var2 * target_temp));
    let var5 = var4 + (var3 * amb_temp); // heater resistance in ohm
    let res_heat = (3.4
        * ((var5 * (4.0 / (4.0 + res_heat_range)) * (1.0 / (1.0 + (res_heat_val * 0.002)))) - 25.0))
        as u8;
    res_heat
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_gas_wait() {
        dbg!(calc_gas_wait_time(100));
    }
}
