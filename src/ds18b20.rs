use std::io::{BufRead, BufReader};
use std::fs;
use std::path::PathBuf;

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

fn find_dir_with_prefix(root_dir: &str, prefix: u32) -> Option<PathBuf> {

    if let Ok(entries) = fs::read_dir(&root_dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();

                if path.is_dir() {
                    if let Some(file_name) = path.file_name() {
                        if let Some((num_str, _)) = file_name.to_str().unwrap().split_once('-') {
                            if let Ok(num) = num_str.parse::<u32>() {
                                if num == prefix {
                                    return Some(path.clone());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    None
}


 #[derive(Debug)]
pub enum SensorError{
    NotFound,
}
pub struct DS18B20{
    sensor_path: PathBuf,
}

impl DS18B20{
    pub fn init() -> Result<DS18B20, SensorError> {
        let root_dir = if is_rpi() {
            "/sys/bus/w1/devices/"
        } else{
            "./test/debug_path/"
        }; // replace this with the root directory you want to start from
        let onewire_address= 28;

        match find_dir_with_prefix(root_dir, onewire_address) {
            Some(path) => {
                println!("{}", path.display());
                Ok( DS18B20{
                    sensor_path: path,
                })
            }
            None => {
                println!("No matching directory found");
                Err(SensorError::NotFound)
            }
        }
    }

    pub fn read_temperture(&self) -> Result<f64, SensorError> {
        let sensor_file_path = self.sensor_path.join("w1_slave");
        // Open file and create buffered reader
        let file = fs::File::open(sensor_file_path).unwrap();
        let reader = BufReader::new(file);

        // Read second line
        let mut line_iter = reader.lines();
        line_iter.next(); // Skip first line
        let second_line = line_iter.next().expect("File has less than 2 lines").unwrap();

        // Extract integer value after "t="
        let t_index = second_line.find("t=").expect("Second line does not contain 't='");
        let integer_str = &second_line[(t_index + 2)..];
        let integer = integer_str.parse::<i32>().expect("Failed to parse integer");

        // Print integer value

        Ok((integer as f64) / 1000.0)
    }
}