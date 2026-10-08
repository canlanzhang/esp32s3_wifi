use anyhow::Result;


use std::sync::{
    Arc,
    Mutex,
};



use esp_idf_hal::i2c::I2cDriver;



#[derive(Clone)]
pub struct Sht31Sensor {


    inner:
        Arc<Mutex<I2cDriver<'static>>>,

}



impl Sht31Sensor {


    pub fn new(

        i2c:I2cDriver<'static>

    )
    ->Result<Self>
    {


        Ok(
            Self{

                inner:
                    Arc::new(
                        Mutex::new(i2c)
                    )

            }
        )

    }




    pub fn read(
        &self
    )
    ->Result<(f32,f32)>
    {

        // TODO:
        // SHT31读取

        Ok(
            (
                25.0,
                60.0
            )
        )

    }

}