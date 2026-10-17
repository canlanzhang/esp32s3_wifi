use anyhow::Result;
use std::sync::{Arc, Mutex};

use esp_idf_hal::gpio::{Output, PinDriver};

#[derive(Clone)]
pub struct IrLed {
    pin: Arc<Mutex<PinDriver<'static, Output>>>,
    state: Arc<Mutex<bool>>,
}

impl IrLed {
    pub fn new(pin: PinDriver<'static, Output>) -> Self {
        let pin = Arc::new(Mutex::new(pin));
        let state = Arc::new(Mutex::new(false));

        // 默认关闭：GPIO LOW 亮，HIGH 灭的 LED 需要输出 HIGH
        {
            let mut led = pin.lock().unwrap();
            led.set_high().unwrap();
        }

        Self { pin, state }
    }

    pub fn on(&self) -> Result<()> {
        let mut led = self.pin.lock().unwrap();

        // 外接 LED：GPIO HIGH 点亮
        led.set_high()?;

        *self.state.lock().unwrap() = true;

        log::info!("GPIO17 LED ON");
        Ok(())
    }

    pub fn off(&self) -> Result<()> {
        let mut led = self.pin.lock().unwrap();

        // 外接 LED：GPIO LOW 熄灭
        led.set_low()?;

        *self.state.lock().unwrap() = false;

        log::info!("GPIO17 LED OFF");
        Ok(())
    }

    pub fn is_on(&self) -> bool {
        *self.state.lock().unwrap()
    }
}