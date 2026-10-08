use anyhow::Result;

use std::sync::{Arc, Mutex};

use esp_idf_hal::gpio::{Output, PinDriver};

#[derive(Clone)]
pub struct Led {
    pin: Arc<Mutex<PinDriver<'static, Output>>>,

    state: Arc<Mutex<bool>>,
}

impl Led {
    pub fn new(pin: PinDriver<'static, Output>) -> Self {
        let pin = Arc::new(Mutex::new(pin));

        let state = Arc::new(Mutex::new(false));

        // 默认关闭
        {
            let mut led = pin.lock().unwrap();

            // GPIO LOW亮
            // HIGH灭
            led.set_high().unwrap();
        }

        Self { pin, state }
    }

    pub fn on(&self) -> Result<()> {
        let mut led = self.pin.lock().unwrap();

        led.set_low()?;

        *self.state.lock().unwrap() = true;

        Ok(())
    }

    pub fn off(&self) -> Result<()> {
        let mut led = self.pin.lock().unwrap();

        led.set_high()?;

        *self.state.lock().unwrap() = false;

        Ok(())
    }

    pub fn is_on(&self) -> bool {
        *self.state.lock().unwrap()
    }
}
