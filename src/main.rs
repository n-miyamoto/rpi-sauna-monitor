use std::thread;
use std::time;

use ambient_rust::{Ambient, AmbientPayload};
use slack_morphism::prelude::*;
use rsb_derive::Builder;
use futures::{future};
use chrono::{Local, Timelike};

mod secrets;
mod sht30;
mod ds18b20;
mod util;

struct SaunaMonitor{
    sht30: sht30::SHT30,
    ds18b: ds18b20::DS18B20,
    ambient: Ambient,
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

    let formatted_payload = format!("time: {:02}:{:02}, sauna: {:.1}°C, {:.1}%, water: {:.1}°C",
        Local::now().hour(),
        Local::now().minute(),
        payload.d2.unwrap(),
        payload.d3.unwrap(),
        payload.d1.unwrap(), 
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
    if util::is_rpi() {
        println!("target is raspberry pi!!!");
    }else {
        println!("target is not raspberry pi. send dummy data.");
    }

    let interval_ms = get_interval_ms();
    println!("Interval = {} [ms]", interval_ms);
    let sleep_time = time::Duration::from_millis(interval_ms);
    let mut sm = SaunaMonitor {
        sht30 : sht30::SHT30::init(),
        ds18b : ds18b20::DS18B20::init().unwrap(),
        ambient: Ambient::new(secrets::ambient::CHANNEL_ID, String::from(secrets::ambient::WRITE_KEY)),
    };

    loop {
        run(&mut sm).await;
        thread::sleep(sleep_time);
    }
}
