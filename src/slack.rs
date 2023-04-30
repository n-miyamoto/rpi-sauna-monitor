use slack_morphism::prelude::*;
use rsb_derive::Builder;

pub async fn post_slack_start_message(slack_token: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = SlackClient::new(SlackClientHyperConnector::new());
    let token_value: SlackApiTokenValue = slack_token.into();
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

pub async fn post_slack_simple_message(msg: String, slack_token: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let client = SlackClient::new(SlackClientHyperConnector::new());
    let token_value: SlackApiTokenValue = slack_token.into();
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
