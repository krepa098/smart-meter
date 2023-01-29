use std::time::{Duration, Instant};

use anyhow::{bail, Result};
use num_enum::TryFromPrimitive;

use crate::bme680;

mod sys {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(unused)]

    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

// additional information:
// https://community.bosch-sensortec.com/varuj77995/attachments/varuj77995/bst_community-mems-forum/1228/1/BST-BME680-Integration-Guide-AN008-47.pdf
pub fn init() -> Result<()> {
    unsafe {
        let ret = sys::bsec_init();
        if ret != sys::bsec_library_return_t_BSEC_OK {
            bail!("Init failed: {}", ret);
        }
    };

    Ok(())
}

pub fn do_steps(inputs: &[Input]) -> Result<Outputs> {
    let mut outputs: [sys::bsec_output_t; sys::BSEC_NUMBER_OUTPUTS as usize] =
        unsafe { std::mem::zeroed() };
    let mut outputs_len: u8 = outputs.len() as u8;

    unsafe {
        let ret = sys::bsec_do_steps(
            inputs.as_ptr(),
            inputs.len() as u8,
            outputs.as_mut_ptr(),
            &mut outputs_len as *mut u8,
        );

        if ret != sys::bsec_library_return_t_BSEC_OK {
            bail!("DoSteps failed: {}", ret);
        }
    }

    let mut po = Outputs::default();
    for o in &outputs[0..outputs_len as usize] {
        if let Ok(sensor_type) = VirtualSensor::try_from(o.sensor_id as u32) {
            match sensor_type {
                VirtualSensor::RawGas => po.raw_gas = Some(Output::from_raw(o)),
                VirtualSensor::RawHumidity => po.raw_humidity = Some(Output::from_raw(o)),
                VirtualSensor::RawPressure => po.raw_pressure = Some(Output::from_raw(o)),
                VirtualSensor::RawTemperature => po.raw_temperature = Some(Output::from_raw(o)),
                VirtualSensor::IAQ => po.iaq = Some(Output::from_raw(o)),
                VirtualSensor::StaticIAQ => po.static_iaq = Some(Output::from_raw(o)),
                VirtualSensor::Co2 => po.co2 = Some(Output::from_raw(o)),
                VirtualSensor::Voc => po.voc = Some(Output::from_raw(o)),
                VirtualSensor::StabilizationStatus => {
                    po.stabilization_status = Some(Output::from_raw(o))
                }
                VirtualSensor::RunInStatus => po.run_in_status = Some(Output::from_raw(o)),
                VirtualSensor::HeatCompensatedTemperature => {
                    po.heat_compensated_temperature = Some(Output::from_raw(o))
                }
                VirtualSensor::HeatCompensatedHumidity => {
                    po.heat_compensated_humidity = Some(Output::from_raw(o))
                }
                VirtualSensor::GasEstimate1 => po.gas_estimate1 = Some(Output::from_raw(o)),
                VirtualSensor::GasEstimate2 => po.gas_estimate2 = Some(Output::from_raw(o)),
                VirtualSensor::GasEstimate3 => po.gas_estimate3 = Some(Output::from_raw(o)),
                VirtualSensor::GasEstimate4 => po.gas_estimate4 = Some(Output::from_raw(o)),
                _ => (),
            }
        }
    }

    Ok(po)
}

pub fn update_subscription(
    virt_sensors: &[VirtualSensorConfiguration],
) -> Result<Vec<SensorConfiguration>> {
    let mut required_sensors: [sys::bsec_sensor_configuration_t;
        sys::BSEC_MAX_PHYSICAL_SENSOR as usize] = unsafe { std::mem::zeroed() };
    let mut len: u8 = sys::BSEC_MAX_PHYSICAL_SENSOR as u8;

    // requested sensors are always virtual
    let requested_virtual_sensors: Vec<_> = virt_sensors.iter().map(|s| s.to_raw()).collect();

    unsafe {
        let ret = sys::bsec_update_subscription(
            requested_virtual_sensors.as_ptr(),
            requested_virtual_sensors.len() as u8,
            required_sensors.as_mut_ptr(),
            &mut len as *mut u8,
        );

        if ret != sys::bsec_library_return_t_BSEC_OK {
            bail!("Cannot update subscription: {}", ret);
        }
    }

    // required sensors are always physical
    let required_sensors = required_sensors[0..len as usize]
        .iter()
        .map(|sc| SensorConfiguration::from_raw_phys(sc))
        .collect();

    Ok(required_sensors)
}

pub fn sensor_control(ts: Duration, sensor: &mut bme680::Device) -> Result<(Outputs, Duration)> {
    let ts_ns = ts.as_nanos() as u64;
    let mut sensor_settings = unsafe { std::mem::zeroed::<sys::bsec_bme_settings_t>() };
    let ret = unsafe { sys::bsec_sensor_control(ts_ns as i64, &mut sensor_settings) };
    if ret != sys::bsec_library_return_t_BSEC_OK {
        bail!("Sensor control error: {}", ret);
    }

    // Note: heater_*_profile are for parallel mode on bme688
    let config = bme680::Config {
        t_oversampling: bme680::Oversampling::try_from_primitive(
            sensor_settings.temperature_oversampling,
        )
        .unwrap(),
        p_oversampling: bme680::Oversampling::try_from_primitive(
            sensor_settings.pressure_oversampling,
        )
        .unwrap(),
        h_oversampling: bme680::Oversampling::try_from_primitive(
            sensor_settings.humidity_oversampling,
        )
        .unwrap(),
        iir_filter: bme680::IIRFilterSize::Fc0,
        gas_heater_temp: sensor_settings.heater_temperature as f32,
        gas_heater_duration_ms: sensor_settings.heater_duration,
        gas_enabled: sensor_settings.run_gas == 1,
        gas_profile: 0,
    };

    if sensor_settings.trigger_measurement == 1 {
        sensor.setup(&config)?;
        sensor.trigger_measurement()?;
    }

    let mes_sleep = std::time::Duration::from_millis(150 + sensor_settings.heater_duration as u64);
    std::thread::sleep(mes_sleep);

    if sensor.is_busy()? {
        bail!("not enough time");
    }

    let mes = sensor.read_measurements()?;

    // feed data to do_steps
    let mut bsec_inputs = heapless::Vec::<Input, 4>::new();
    if (sensor_settings.process_data & sys::PROCESS_GAS) > 0 {
        bsec_inputs
            .push(Input::new(
                ts_ns,
                Sensor::Phyical(PhysicalSensor::Gasresistor),
                mes.gas_res.unwrap(),
            ))
            .unwrap();
    }
    if (sensor_settings.process_data & sys::PROCESS_HUMIDITY) > 0 {
        bsec_inputs
            .push(Input::new(
                ts_ns,
                Sensor::Phyical(PhysicalSensor::Humidity),
                mes.humidity.unwrap(),
            ))
            .unwrap();
    }
    if (sensor_settings.process_data & sys::PROCESS_PRESSURE) > 0 {
        bsec_inputs
            .push(Input::new(
                ts_ns,
                Sensor::Phyical(PhysicalSensor::Pressure),
                mes.pressure.unwrap(),
            ))
            .unwrap();
    }
    if (sensor_settings.process_data & sys::PROCESS_TEMPERATURE) > 0 {
        bsec_inputs
            .push(Input::new(
                ts_ns,
                Sensor::Phyical(PhysicalSensor::Temperature),
                mes.temperature.unwrap(),
            ))
            .unwrap();
    }

    let outputs = do_steps(&bsec_inputs)?;

    Ok((
        outputs,
        Duration::from_nanos(sensor_settings.next_call as u64),
    ))
}

pub fn version() -> Result<[u8; 4]> {
    let mut version = unsafe { std::mem::zeroed::<sys::bsec_version_t>() };
    let ret = unsafe { sys::bsec_get_version(&mut version) };
    if ret != sys::bsec_library_return_t_BSEC_OK {
        bail!("Cannot get version: {}", ret);
    }

    Ok([
        version.major,
        version.minor,
        version.major_bugfix,
        version.minor_bugfix,
    ])
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, TryFromPrimitive)]
#[allow(unused)]
pub enum PhysicalSensor {
    // in Pa
    Pressure = sys::bsec_physical_sensor_t_BSEC_INPUT_PRESSURE,
    // in percent
    Humidity = sys::bsec_physical_sensor_t_BSEC_INPUT_HUMIDITY,
    // in °C
    Temperature = sys::bsec_physical_sensor_t_BSEC_INPUT_TEMPERATURE,
    // in ohm
    Gasresistor = sys::bsec_physical_sensor_t_BSEC_INPUT_GASRESISTOR,
    // an offset for any heating induced by other components
    HeatSource = sys::bsec_physical_sensor_t_BSEC_INPUT_HEATSOURCE,
    // 0 - Normal, 1 - Event1, 2 - Event2
    DisableBaselineTracker = sys::bsec_physical_sensor_t_BSEC_INPUT_DISABLE_BASELINE_TRACKER,
    // provides information about the state of the profile (1-9), gas index
    ProfilePart = sys::bsec_physical_sensor_t_BSEC_INPUT_PROFILE_PART,

    Unknown,
}

#[derive(Debug, Clone, Copy, TryFromPrimitive)]
#[repr(u32)]
#[allow(unused)]
pub enum VirtualSensor {
    IAQ = sys::bsec_virtual_sensor_t_BSEC_OUTPUT_IAQ,
    StaticIAQ = sys::bsec_virtual_sensor_t_BSEC_OUTPUT_STATIC_IAQ,
    Co2 = sys::bsec_virtual_sensor_t_BSEC_OUTPUT_CO2_EQUIVALENT,
    Voc = sys::bsec_virtual_sensor_t_BSEC_OUTPUT_BREATH_VOC_EQUIVALENT,
    RawPressure = sys::bsec_virtual_sensor_t_BSEC_OUTPUT_RAW_PRESSURE,
    RawHumidity = sys::bsec_virtual_sensor_t_BSEC_OUTPUT_RAW_HUMIDITY,
    RawTemperature = sys::bsec_virtual_sensor_t_BSEC_OUTPUT_RAW_TEMPERATURE,
    RawGas = sys::bsec_virtual_sensor_t_BSEC_OUTPUT_RAW_GAS,
    StabilizationStatus = sys::bsec_virtual_sensor_t_BSEC_OUTPUT_STABILIZATION_STATUS,
    RunInStatus = sys::bsec_virtual_sensor_t_BSEC_OUTPUT_RUN_IN_STATUS,
    HeatCompensatedTemperature =
        sys::bsec_virtual_sensor_t_BSEC_OUTPUT_SENSOR_HEAT_COMPENSATED_TEMPERATURE,
    HeatCompensatedHumidity =
        sys::bsec_virtual_sensor_t_BSEC_OUTPUT_SENSOR_HEAT_COMPENSATED_HUMIDITY,
    GasPercentage = sys::bsec_virtual_sensor_t_BSEC_OUTPUT_GAS_PERCENTAGE,
    GasEstimate1 = sys::bsec_virtual_sensor_t_BSEC_OUTPUT_GAS_ESTIMATE_1,
    GasEstimate2 = sys::bsec_virtual_sensor_t_BSEC_OUTPUT_GAS_ESTIMATE_2,
    GasEstimate3 = sys::bsec_virtual_sensor_t_BSEC_OUTPUT_GAS_ESTIMATE_3,
    GasEstimate4 = sys::bsec_virtual_sensor_t_BSEC_OUTPUT_GAS_ESTIMATE_4,
    RawGasIndex = sys::bsec_virtual_sensor_t_BSEC_OUTPUT_RAW_GAS_INDEX,
}

pub type Input = sys::bsec_input_t;

impl Input {
    pub fn new(ts_ns: u64, sensor: Sensor, signal: f32) -> Self {
        Self {
            time_stamp: ts_ns as i64,
            signal,
            signal_dimensions: 1,
            sensor_id: match sensor {
                Sensor::Virtual(v) => v as u8,
                Sensor::Phyical(p) => p as u8,
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
#[allow(unused)]
pub enum SampleRate {
    Disabled,
    Ulp,
    Cont,
    Lp,
}

impl SampleRate {
    fn to_raw(&self) -> f32 {
        match self {
            SampleRate::Disabled => sys::BSEC_SAMPLE_RATE_DISABLED as f32,
            SampleRate::Ulp => sys::BSEC_SAMPLE_RATE_ULP as f32,
            SampleRate::Cont => sys::BSEC_SAMPLE_RATE_CONT as f32,
            SampleRate::Lp => sys::BSEC_SAMPLE_RATE_LP as f32,
        }
    }
}

#[derive(Debug)]
pub struct VirtualSensorConfiguration {
    pub sample_rate: SampleRate,
    pub sensor: VirtualSensor,
}

impl VirtualSensorConfiguration {
    fn to_raw(&self) -> sys::bsec_sensor_configuration_t {
        sys::bsec_sensor_configuration_t {
            sample_rate: self.sample_rate.to_raw(),
            sensor_id: self.sensor as u8,
        }
    }
}

#[derive(Debug)]
pub struct SensorConfiguration {
    pub sample_rate: f32,
    pub sensor: Sensor,
}

impl SensorConfiguration {
    fn from_raw_virt(r: &sys::bsec_sensor_configuration_t) -> Self {
        let sensor = {
            if let Ok(s) = VirtualSensor::try_from_primitive(r.sensor_id as u32) {
                Sensor::Virtual(s)
            } else {
                dbg!(r.sensor_id);
                unimplemented!()
            }
        };

        Self {
            sample_rate: r.sample_rate,
            sensor,
        }
    }

    fn from_raw_phys(r: &sys::bsec_sensor_configuration_t) -> Self {
        let sensor = {
            if let Ok(s) = PhysicalSensor::try_from_primitive(r.sensor_id as u32) {
                Sensor::Phyical(s)
            } else {
                dbg!(r.sensor_id);
                Sensor::Phyical(PhysicalSensor::Unknown)
            }
        };

        Self {
            sample_rate: r.sample_rate,
            sensor,
        }
    }
}

#[derive(Debug)]
pub enum Sensor {
    Virtual(VirtualSensor),
    Phyical(PhysicalSensor),
}

impl Sensor {
    fn from_raw(sensor_id: u8) -> Self {
        if let Ok(s) = PhysicalSensor::try_from_primitive(sensor_id as u32) {
            Sensor::Phyical(s)
        } else if let Ok(s) = VirtualSensor::try_from_primitive(sensor_id as u32) {
            Sensor::Virtual(s)
        } else {
            dbg!(sensor_id);
            unimplemented!()
        }
    }
}

#[derive(Debug, Default)]
pub struct Outputs {
    pub raw_gas: Option<Output>,
    pub raw_humidity: Option<Output>,
    pub raw_temperature: Option<Output>,
    pub raw_pressure: Option<Output>,
    pub iaq: Option<Output>,
    pub static_iaq: Option<Output>,
    pub co2: Option<Output>,
    pub voc: Option<Output>,
    pub stabilization_status: Option<Output>,
    pub run_in_status: Option<Output>,
    pub heat_compensated_temperature: Option<Output>,
    pub heat_compensated_humidity: Option<Output>,
    pub gas_estimate1: Option<Output>,
    pub gas_estimate2: Option<Output>,
    pub gas_estimate3: Option<Output>,
    pub gas_estimate4: Option<Output>,
}

#[derive(Debug)]
pub struct Output {
    pub timestamp: u64,
    pub signal: f32,
    pub sensor: VirtualSensor,
    pub accuracy: Accuracy,
}

impl Output {
    fn from_raw(r: &sys::bsec_output_t) -> Self {
        Self {
            timestamp: r.time_stamp as u64,
            signal: r.signal,
            sensor: VirtualSensor::try_from_primitive(r.sensor_id as u32).unwrap(),
            accuracy: Accuracy::try_from_primitive(r.accuracy).unwrap(),
        }
    }
}

#[derive(Debug, Clone, Copy, TryFromPrimitive)]
#[repr(u8)]
pub enum Accuracy {
    Unreliable = 0,
    Low = 1,
    Medium = 2,
    High = 3,
}
