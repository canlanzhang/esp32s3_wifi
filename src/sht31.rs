use anyhow::Result;

use std::sync::{Arc, Mutex};

use esp_idf_hal::i2c::I2cDriver;

use sht31::{
    mode::{Sht31Reader, SimpleSingleShot},
    SHT31,
};

#[derive(Clone)]
pub struct Sht31Sensor {
    sensor: Arc<Mutex<SHT31<SimpleSingleShot, I2cDriver<'static>>>>,
}

impl Sht31Sensor {
    pub fn new(i2c: I2cDriver<'static>) -> Result<Self> {
        let sensor = SHT31::new(i2c);

        Ok(Self {
            sensor: Arc::new(Mutex::new(sensor)),
        })
    }

    pub fn read(&self) -> Result<(f32, f32)> {
        let mut sensor = self.sensor.lock().unwrap();

        let data = sensor.read()?;

        Ok((data.temperature, data.humidity))
    }
}
