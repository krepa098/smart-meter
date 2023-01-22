use anyhow::{bail, Result};
use num_enum::TryFromPrimitive;

mod sys {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(unused)]

    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

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
    let mut outputs_len: u8 = 0;

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
                VirtualSensor::IAQ => po.iaq = Some(*o),
                VirtualSensor::StaticIAQ => po.static_iaq = Some(*o),
                VirtualSensor::Co2 => po.co2 = Some(*o),
                VirtualSensor::Voc => po.voc = Some(*o),
                VirtualSensor::StabilizationStatus => po.stabilization_status = Some(*o),
                VirtualSensor::RunInStatus => po.run_in_status = Some(*o),
                VirtualSensor::HeatCompensatedTemperature => {
                    po.heat_compensated_temperature = Some(*o)
                }
                VirtualSensor::HeatCompensatedHumidity => po.heat_compensated_humidity = Some(*o),
            }
        }
    }

    Ok(po)
}

pub fn update_subscription(
    virt_sensors: &[VirtualSensorConfiguration],
) -> Result<Vec<PhysicalSensorConfiguration>> {
    let mut required_sensors: [sys::bsec_sensor_configuration_t;
        sys::BSEC_MAX_PHYSICAL_SENSOR as usize] = unsafe { std::mem::zeroed() };
    let mut len: u8 = 0;

    let requested_virtual_sensors: Vec<_> = virt_sensors.iter().map(|s| s.0.to_owned()).collect();

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

    let required_sensors = required_sensors[0..len as usize]
        .iter()
        .map(|sc| PhysicalSensorConfiguration(*sc))
        .collect();

    Ok(required_sensors)
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, TryFromPrimitive)]
#[allow(unused)]
pub enum PhysicalSensor {
    Pressure = sys::bsec_physical_sensor_t_BSEC_INPUT_PRESSURE,
    Humidity = sys::bsec_physical_sensor_t_BSEC_INPUT_HUMIDITY,
    Temperature = sys::bsec_physical_sensor_t_BSEC_INPUT_TEMPERATURE,
    Gasresistor = sys::bsec_physical_sensor_t_BSEC_INPUT_GASRESISTOR,
}

#[derive(Debug, Clone, Copy, TryFromPrimitive)]
#[repr(u32)]
#[allow(unused)]
pub enum VirtualSensor {
    IAQ = sys::bsec_virtual_sensor_t_BSEC_OUTPUT_IAQ,
    StaticIAQ = sys::bsec_virtual_sensor_t_BSEC_OUTPUT_STATIC_IAQ,
    Co2 = sys::bsec_virtual_sensor_t_BSEC_OUTPUT_CO2_EQUIVALENT,
    Voc = sys::bsec_virtual_sensor_t_BSEC_OUTPUT_BREATH_VOC_EQUIVALENT,
    StabilizationStatus = sys::bsec_virtual_sensor_t_BSEC_OUTPUT_STABILIZATION_STATUS,
    RunInStatus = sys::bsec_virtual_sensor_t_BSEC_OUTPUT_RUN_IN_STATUS,
    HeatCompensatedTemperature =
        sys::bsec_virtual_sensor_t_BSEC_OUTPUT_SENSOR_HEAT_COMPENSATED_TEMPERATURE,
    HeatCompensatedHumidity =
        sys::bsec_virtual_sensor_t_BSEC_OUTPUT_SENSOR_HEAT_COMPENSATED_HUMIDITY,
}

pub type Input = sys::bsec_input_t;

impl Input {
    pub fn new(sensor: PhysicalSensor, signal: f32) -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};

        let since_epoch = {
            let start = SystemTime::now();
            let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap();
            since_the_epoch.as_nanos() as i64
        };

        Self {
            time_stamp: since_epoch,
            signal,
            signal_dimensions: 1,
            sensor_id: sensor as u8,
        }
    }
}

pub type Output = sys::bsec_output_t;

pub struct VirtualSensorConfiguration(sys::bsec_sensor_configuration_t);

pub struct PhysicalSensorConfiguration(sys::bsec_sensor_configuration_t);

#[derive(Debug, Default)]
pub struct Outputs {
    pub iaq: Option<Output>,
    pub static_iaq: Option<Output>,
    pub co2: Option<Output>,
    pub voc: Option<Output>,
    pub stabilization_status: Option<Output>,
    pub run_in_status: Option<Output>,
    pub heat_compensated_temperature: Option<Output>,
    pub heat_compensated_humidity: Option<Output>,
}
