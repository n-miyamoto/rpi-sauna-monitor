use rppal::i2c::I2c;
use std::{thread, time};
use ambient_rust::{Ambient, AmbientPayload};

mod secrets;

fn is_rpi() -> bool {
    if !cfg!(target_arch="arm") {
        return false;
    }else if !cfg!(target_os = "linux") {
        return false;
    }else if !cfg!(target_env= "gnu") {
        return false;
    }

    true
}

#[derive(Debug)]
enum SensorError{
    NotFound,
}

struct SHT30 {
    i2c : Option<I2c>,
}

const SHT30_ADDR : u16 = 0x44;
const SHT30_MODE : u8 = 0x2C;
const SHT30_HIGH : u8 = 0x06;
const SHT30_READ : u8 = 0x00;

impl SHT30{

    fn init () -> SHT30 {
        //non-raspi case
        if !is_rpi() {return SHT30 { i2c: None, };};

        let mut i2c = I2c::new().unwrap();
        i2c.set_slave_address(SHT30_ADDR).unwrap(); 
        i2c.block_write(
            SHT30_MODE as u8,
            &[SHT30_HIGH as u8],
        ).unwrap();

        SHT30 {
            i2c : Some(i2c),
        }
    }

    fn read_temperture(&mut self) -> Result<f64, SensorError> {
        //non-raspi case
        if !is_rpi() {
            return Ok(12.3); //dummy data
        };

        // read sensor.
        let mut reg = [0u8; 6];
        self.i2c.as_mut().unwrap().block_read(SHT30_READ, &mut reg).unwrap();

        let temp : u16 = (reg[0] as u16) << 8 | reg[1] as u16;
        let temp : f64 = -45.0 + 175.0 * (temp as f64) / 65535.0 ;
        Ok(temp)
    }

    fn read_humidity(&mut self) -> Result<f64, SensorError> {
        //non-raspi case
        if !is_rpi() {
            return Ok(45.6); //dummy data
        };

        // read sensor.
        let mut reg = [0u8; 6];
        self.i2c.as_mut().unwrap().block_read(SHT30_READ, &mut reg).unwrap();

        let humid : u16 = (reg[3] as u16) << 8 | reg[4] as u16;
        let humid : f64 = 100.0 * humid as f64 / 65535.0;
        Ok(humid)
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
        //non-raspi case
        if !cfg!(arm) || !cfg!(linux) {
            return DS18B20{ onewire_address: 28};
        };

        // TODO: initialize one-wire
        // TODO: initialize DS18B20 sensor
        DS18B20 {
            onewire_address : 28,
        }
    }

    fn read_temperture(&self) -> Result<f64, SensorError> {
        //non-raspi case
        if !cfg!(arm) || !cfg!(linux) {
            return Ok(90.12);
        };

        Ok(90.12)
    }
}

fn run(sauna_monitor : &mut SaunaMonitor){
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
    if is_rpi() {
        println!("target is raspberry pi!!!");
    }else {
        println!("target is not raspberry pi. send dummy data.");
    }

    let interval_ms = 5_000;
    let sleep_time = time::Duration::from_millis(interval_ms);
    let mut sm = SaunaMonitor {
        sht30 : SHT30::init(),
        ds18b : DS18B20::init(),
        ambient: Ambient::new(secrets::ambient::CHANNEL_ID, String::from(secrets::ambient::WRITE_KEY)),
    };

    loop {
        run(&mut sm);

        thread::sleep(sleep_time);
    }
}
