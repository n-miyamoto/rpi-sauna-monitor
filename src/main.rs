use rppal::i2c::I2c;
use std::{thread, time};
use ambient_rust::{Ambient, AmbientPayload};

mod secrets;

#[derive(Debug)]
enum SensorError{
    NotFound,
}

struct SHT30 {
    i2c_address : u32, 
}

impl SHT30{
    fn init () -> SHT30 {
        // TODO : init i2c using rppal, SHT30 sensor.
        SHT30 {
            i2c_address : 44,
        }
    }

    fn read_temperture(&self) -> Result<f64, SensorError> {
        // TODO : read STH30 sensor temperture data and return value
        Ok(12.34)
    }

    fn read_humidity(&self) -> Result<f64, SensorError> {
        // TODO : read STH30 sensor humidity data and return value
        Ok(56.78)
    }
}

struct SaunaMonitor{
    sht30: SHT30,
    ds18b: DS18B20,
    ambient: Ambient,
}

struct DS18B20{
    onewire_address : u64,
}
impl DS18B20{
    fn init() -> DS18B20 {

        DS18B20 {
            onewire_address : 28,
        }
    }

    fn read_temperture(&self) -> Result<f64, SensorError> {
        Ok(90.12)
    }
}

fn run(sauna_monitor : &SaunaMonitor){
    let payload = AmbientPayload {
        //created: Some(Utc::now()), Persing chrono::DataTime is not supported yes.
        created: None,
        d1: Some(sauna_monitor.ds18b.read_temperture().unwrap()),
        d2: Some(sauna_monitor.sht30.read_temperture().unwrap()),
        d3: Some(sauna_monitor.sht30.read_humidity().unwrap()),
        d4: None,
        d5: None,
        d6: None,
        d7: None,
        d8: None,
    };

    println!("{:?}", payload);

    let response = sauna_monitor.ambient.send(&payload, None);
    match &response{
        Ok(res) =>  {
            println!("{:?}", res.status());
        },
        Err(error) => {
            panic!("Http post failled.: {:?}", error);
        }
    }

}

fn main() {
    println!("rpi-sauna-monitor\nHello, world!");

    let interval_ms = 1_000;
    let sleep_time = time::Duration::from_millis(interval_ms);
    let sm = SaunaMonitor {
        sht30 : SHT30::init(),
        ds18b : DS18B20::init(),
        ambient: Ambient::new(secrets::ambient::CHANNEL_ID, String::from(secrets::ambient::WRITE_KEY)),
    };

    loop {
        run(&sm);

        thread::sleep(sleep_time);
    }

}
