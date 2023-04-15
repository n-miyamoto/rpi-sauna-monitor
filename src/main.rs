use std::thread;
use std::time;
use std::path::PathBuf;
use std::io::{BufRead, BufReader};
use std::fs;

use ambient_rust::{Ambient, AmbientPayload};
use rppal::i2c::I2c;
use slack_morphism::prelude::*;
use rsb_derive::Builder;
use futures::{future};

mod secrets;

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
enum SensorError{
    NotFound,
}

struct SHT30 {
    i2c : Option<I2c>,
}

impl SHT30{
    const ADDR : u16 = 0x44;
    const MODE : u8 = 0x2C;
    const HIGH : u8 = 0x06;
    const READ : u8 = 0x00;
    const WAIT_TIME_MS: u64 = 200;

    fn init () -> SHT30 {
        //non-raspi case
        if !is_rpi() {return SHT30 { i2c: None, };};

        let mut i2c = I2c::new().unwrap();
        i2c.set_slave_address(SHT30::ADDR).unwrap(); 

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

    fn read_humidity(&mut self) -> Result<f64, SensorError> {
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

struct SaunaMonitor{
    sht30: SHT30,
    ds18b: DS18B20,
    ambient: Ambient,
}

struct DS18B20{
    sensor_path: PathBuf,
}

impl DS18B20{
    fn init() -> Result<DS18B20, SensorError> {
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

    fn read_temperture(&self) -> Result<f64, SensorError> {
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


pub fn config_env_var(name: &str) -> Result<String, String> {
    std::env::var(name).map_err(|e| format!("{}: {}", name, e))
}

async fn post_slack_start_message() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = SlackClient::new(SlackClientHyperConnector::new());
    let token_value: SlackApiTokenValue = secrets::slack::SLACK_TEST_TOKEN.into();
    let token: SlackApiToken = SlackApiToken::new(token_value);
    let session = client.open_session(&token);

    let message = WelcomeMessageTemplateParams::new("sauna-monitor".into());

    let post_chat_req =
        SlackApiChatPostMessageRequest::new("#sauna".into(), message.render_template());

    session.chat_post_message(&post_chat_req).await?;

    Ok(())
}

#[derive(Debug, Clone, Builder)]
pub struct WelcomeMessageTemplateParams {
    pub user_id: SlackUserId,
}

impl SlackMessageTemplate for WelcomeMessageTemplateParams {
    fn render_template(&self) -> SlackMessageContent {
        SlackMessageContent::new()
            .with_text(format!("Hey {}", self.user_id.to_slack_format()))
            .with_blocks(slack_blocks![
                some_into(SlackHeaderBlock::new(pt!("RPi Sauna Monitor"))),
                some_into(
                    SlackSectionBlock::new()
                        .with_text(md!("Hey {} rpi sauna nomitor started working. url: https://ambidata.io/bd/board.html?id=18138", self.user_id.to_slack_format()))
                )
            ])
    }
}

async fn post_slack_simple_message(msg: String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = SlackClient::new(SlackClientHyperConnector::new());
    let token_value: SlackApiTokenValue = secrets::slack::SLACK_TEST_TOKEN.into();
    let token: SlackApiToken = SlackApiToken::new(token_value);
    let session = client.open_session(&token);

    let id : SlackUserId = "sauna-monitor".into();
    let message = SlackMessageContent::new()
            .with_text(format!("Hey {}", id.to_slack_format()))
            .with_blocks(slack_blocks![
                some_into(
                    SlackSectionBlock::new()
                        .with_text(md!("{}", msg))
                )
            ]);

    let post_chat_req =
        SlackApiChatPostMessageRequest::new("#sauna".into(), message);

    session.chat_post_message(&post_chat_req).await?;

    Ok(())
}

async fn run(sauna_monitor : &mut SaunaMonitor) {
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

    let formatted_payload = format!("water temp: {:.1}, sauna temp: {:.1}, sauna humid: {:.1}", 
        payload.d1.unwrap(), 
        payload.d2.unwrap(),
        payload.d3.unwrap(),
    );
    println!("{}", formatted_payload);

    let (res_ambient, res_slack) = future::join(
        sauna_monitor.ambient.send(&payload, None),
        post_slack_simple_message(formatted_payload)
    ).await;


    match &res_ambient{
        Ok(res) =>  {
            println!("Http status code : {:?}", res.status());
        },
        Err(error) => {
            println!("Http post failled.: {:?}", error);
        }
    }

    match &res_slack{
        Ok(_) =>  {
            println!("Slack : OK");
        },
        Err(error) => {
            println!("Slack post failed.: {:?}", error);
        }
    }
}

fn get_interval_ms() -> u64{
    if cfg!(test) {
        5_000
    } else if cfg!(debug_assertions) {
        5_000
    } else {
        60_000
    }
}

#[tokio::main]
async fn main(){

    let res_slack = post_slack_start_message().await;
    match &res_slack{
        Ok(_) => println!("Slack : OK"),
        Err(error) => println!("Slack post failed.: {:?}", error),
    }

    println!("rpi-sauna-monitor\nHello, world!");
    if is_rpi() {
        println!("target is raspberry pi!!!");
    }else {
        println!("target is not raspberry pi. send dummy data.");
    }

    let interval_ms = get_interval_ms();
    println!("Interval = {} [ms]", interval_ms);
    let sleep_time = time::Duration::from_millis(interval_ms);
    let mut sm = SaunaMonitor {
        sht30 : SHT30::init(),
        ds18b : DS18B20::init().unwrap(),
        ambient: Ambient::new(secrets::ambient::CHANNEL_ID, String::from(secrets::ambient::WRITE_KEY)),
    };

    loop {
        run(&mut sm).await;
        thread::sleep(sleep_time);
    }
}
