use async_trait::async_trait;
use rand::Rng;
use twitch_irc::message::ServerMessage;

use crate::twitch::{handler::{TwitchMessageHandler, TwitchHandlerError}, bot::MuniBotTwitchIRCClient};

pub struct BonkHandler;

#[async_trait]
impl TwitchMessageHandler for BonkHandler {
    async fn handle_message(
        &mut self,
        client: &MuniBotTwitchIRCClient,
        message: ServerMessage,
    ) -> Result<bool, TwitchHandlerError> {
        let handled = if let ServerMessage::Privmsg(m) = message {
            if let Some(target) = m.message_text.trim().strip_prefix("!bonk ") {
                    // pick a template
                    let template_index = rand::thread_rng().gen_range(0..BONK_TEMPLATES.len());

                    // get message by replacing all {target}s with the sender's name
                    let message = BONK_TEMPLATES[template_index].replace("{target}", target);

                    // and send!
                    self.send_message(client, &m.channel_login, &message)
                        .await.unwrap();

                    true
            } else {
                false
            }
        } else {
            false
        };

        Ok(handled)
    }
}

const BONK_TEMPLATES: [&str; 10] = [
    "{target}, stop being naughty BOP",
    "sorry about this, {target} BOP",
    "{target} >:( BOP",
    "*sigh* this will only hurt a little, {target} BOP",
    "{target} needs a bonk?? BOP",
    "surely you saw this coming, {target}? BOP",
    "here you go, {target} BOP",
    "don't move, {target} BOP",
    "bad {target}, bad! BOP",
    "sounds like you've been naughty, {target} BOP",
];
