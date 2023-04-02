use std::fs;

//const ONEWIRE_DEVFILE_PATH : &str = "/sys/bus/w1/devices/";
const ONEWIRE_DEVFILE_PATH : &str = "./debug/"; //for debug
pub struct OneWire {
    address : u64,
    target_dir: Option<String>,
}

impl OneWire{
    pub fn new(address: u64) -> OneWire {
        let res = fs::read_dir(ONEWIRE_DEVFILE_PATH);
        let mut target_dir: Option<String> = None;
        match res{
            Ok(entries) => {
                for entry in entries{
                    if let Some(filename) = entry.unwrap().file_name().to_str(){
                        if let Some((num_str, _)) = filename.split_once('-') {
                            if let Ok(num) = num_str.parse::<u64>() {
                                println!("{}", num); // replace this with your desired action
                                if num==address{
                                    println!("{}", filename);
                                    target_dir = Some(filename.to_string());
                                }
                            }
                        }
                    }
                }
            },
            Err(_) => {},
        }

        OneWire {
            address, 
            target_dir
        }
    }

    pub fn search() -> Option<u64> {
        let res = fs::read_dir(ONEWIRE_DEVFILE_PATH);
        match res{
            Ok(entries) => {
                for entry in entries{
                    if let Some(filename) = entry.unwrap().file_name().to_str(){
                        if let Some((num_str, _)) = filename.split_once('-') {
                            if let Ok(num) = num_str.parse::<u64>() {
                                println!("{}", num); // replace this with your desired action
                                return Some(num);
                            }
                        }
                    }
                }
            },
            Err(_) => {return None},
        }


        //TODO impl search code
        let onewire_address = 0x1234;
        Some(onewire_address)
    }

    fn is_crc_valid(&self) -> bool{
        //TODO check crc
        true
    }

    pub fn write() {
    }

    pub fn read(&self, data : &mut [u8; 8]){
    }
}

