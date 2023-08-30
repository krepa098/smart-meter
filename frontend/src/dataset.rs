use common::req::{MeasurementRequestResponse, MeasurementType};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Series {
    pub data: Vec<(i64, f32)>,
    pub kind: MeasurementType,
    pub unit: String,
    pub name: String,
    scale: f32,
}

pub type Dataset = HashMap<MeasurementType, Series>;

pub fn dataset_from_request(resp: &MeasurementRequestResponse) -> Dataset {
    let mut dataset = Dataset::new();
    dataset.insert(
        MeasurementType::Temperature,
        Series {
            name: "Temperature".to_owned(),
            unit: "°C".to_owned(),
            kind: MeasurementType::Temperature,
            data: vec![],
            scale: 1.0,
        },
    );
    dataset.insert(
        MeasurementType::Humidity,
        Series {
            name: "Humidity".to_owned(),
            unit: "%".to_owned(),
            kind: MeasurementType::Humidity,
            data: vec![],
            scale: 1.0,
        },
    );
    dataset.insert(
        MeasurementType::Pressure,
        Series {
            name: "Pressure".to_owned(),
            unit: "hPa".to_owned(),
            kind: MeasurementType::Pressure,
            data: vec![],
            scale: 1e-2,
        },
    );
    dataset.insert(
        MeasurementType::AirQuality,
        Series {
            name: "Air Quality".to_owned(),
            unit: "IAQ".to_owned(),
            kind: MeasurementType::AirQuality,
            data: vec![],
            scale: 1.0,
        },
    );
    dataset.insert(
        MeasurementType::BatVoltage,
        Series {
            name: "Battery Voltage".to_owned(),
            unit: "V".to_owned(),
            kind: MeasurementType::BatVoltage,
            data: vec![],
            scale: 1.0,
        },
    );

    for (k, v) in &resp.data {
        if let Ok(meas_type) = MeasurementType::try_from(*k) {
            if dataset.contains_key(&meas_type) {
                let series = dataset.get_mut(&meas_type).unwrap();
                let scale = series.scale;
                dataset.get_mut(&meas_type).unwrap().data = resp
                    .timestamps
                    .iter()
                    .zip(v)
                    .map(|(a, b)| (*a, b.map_or(std::f32::NAN, |v| v * scale)))
                    .collect::<Vec<_>>();
            }
        }
    }

    dataset.insert(
        MeasurementType::DewPoint,
        dew_point(
            &dataset.get(&MeasurementType::Humidity).unwrap().data,
            &dataset.get(&MeasurementType::Temperature).unwrap().data,
        ),
    );

    dataset
}

#[derive(Debug)]
pub struct SeriesStats {
    pub x_min: i64,
    pub x_max: i64,
    pub y_max: f32,
    pub y_min: f32,
}

impl SeriesStats {
    pub fn x_range(&self) -> i64 {
        self.x_max - self.x_min
    }

    pub fn y_range(&self) -> f32 {
        self.y_max - self.y_min
    }
}

pub trait Stats {
    fn stats(&self) -> SeriesStats;
}

impl Stats for Series {
    fn stats(&self) -> SeriesStats {
        let mut x_min = i64::MAX;
        let mut x_max = i64::MIN;
        let mut y_min = f32::MAX;
        let mut y_max = f32::MIN;

        for (x, y) in &self.data {
            if y_max < *y {
                y_max = *y;
                x_max = *x;
            }
            if y_min > *y {
                y_min = *y;
                x_min = *x;
            }
        }

        SeriesStats {
            x_min,
            x_max,
            y_min,
            y_max,
        }
    }
}

pub fn dew_point(rh: &[(i64, f32)], temp: &[(i64, f32)]) -> Series {
    Series {
        data: rh
            .iter()
            .zip(temp)
            .map(|((t, rh), (_, temp))| {
                let a = 17.625;
                let b = 243.04;
                let alpha = (rh / 100.0).ln() + a * temp / (b + temp);
                (*t, (b * alpha) / (a - alpha))
            })
            .collect(),
        kind: MeasurementType::DewPoint,
        unit: "°C".to_string(),
        name: "Dew Point".to_string(),
        scale: 1.0,
    }
}
