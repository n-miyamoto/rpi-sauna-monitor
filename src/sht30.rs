use std::time;
use std::thread;
use rppal::i2c::I2c;

fn is_rpi() -> bool {
    if cfg!(target_arch="arm") && 
       cfg!(target_os="linux") &&
       cfg!(target_env="gnu")
    {
        true
    }else{
        false
    }
}

 #[derive(Debug)]
pub enum SensorError{
    //NotFound,
}
pub struct SHT30 {
    i2c : Option<I2c>,
}

impl SHT30{
    const ADDR : u16 = 0x44;
    const MODE : u8 = 0x2C;
    const HIGH : u8 = 0x06;
    const READ : u8 = 0x00;
    const WAIT_TIME_MS: u64 = 200;

    pub fn init () -> SHT30 {
        //non-raspi case
        if !is_rpi() {return SHT30 { i2c: None, };};

        let mut i2c = I2c::new().unwrap();
        i2c.set_slave_address(SHT30::ADDR).unwrap(); 

        SHT30 {
            i2c : Some(i2c),
        }
    }

    pub fn read_temperture(&mut self) -> Result<f64, SensorError> {
        //non-raspi case
        if !is_rpi() {
            return Ok(12.3); //dummy data
        };

        // read sensor.
        self.i2c.as_mut().unwrap().block_write(
            SHT30::MODE as u8,
            &[SHT30::HIGH as u8],
        ).unwrap();
        let wait_time_ms = time::Duration::from_millis(SHT30::WAIT_TIME_MS);
        thread::sleep(wait_time_ms);

        let mut reg = [0u8; 6];
        self.i2c.as_mut().unwrap().block_read(SHT30::READ, &mut reg).unwrap();
        thread::sleep(wait_time_ms);

        let temp : u16 = (reg[0] as u16) << 8 | reg[1] as u16;
        let temp : f64 = -45.0 + 175.0 * (temp as f64) / 65535.0 ;
        Ok(temp)
    }

    pub fn read_humidity(&mut self) -> Result<f64, SensorError> {
        //non-raspi case
        if !is_rpi() {
            return Ok(45.6); //dummy data
        };

        // read sensor.
        self.i2c.as_mut().unwrap().block_write(
            SHT30::MODE as u8,
            &[SHT30::HIGH as u8],
        ).unwrap();
        let wait_time_ms = time::Duration::from_millis(SHT30::WAIT_TIME_MS);
        thread::sleep(wait_time_ms);

        let mut reg = [0u8; 6];
        self.i2c.as_mut().unwrap().block_read(SHT30::READ, &mut reg).unwrap();
        thread::sleep(wait_time_ms);

        let humid : u16 = (reg[3] as u16) << 8 | reg[4] as u16;
        let humid : f64 = 100.0 * humid as f64 / 65535.0;
        Ok(humid)
    }
}
