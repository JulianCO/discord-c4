mod connect4;
mod protocol;

use connect4::persistency;

use discord::model::{Event, ReactionEmoji, Channel};
use discord::Discord;

use std::env;

use protocol::Request;
use protocol::COLUMN_EMOJI;

// invite through https://discord.com/api/oauth2/authorize?client_id=805143667392118794&scope=bot&permissions=75840

fn main() {
    let discord = Discord::from_bot_token(
        &env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not set in environment"),
    )
    .expect("login failed");

    println!("making a database at /etc/c4-matches/prod_env.sqlite");
    let mut conn =
        persistency::initialize("/etc/c4-matches/prod_env.sqlite").expect("failed to initialize database");

    let (mut connection, _) = discord.connect().expect("connect failed");
    let bot_id = discord.get_current_user().expect("failed to find self").id;
    println!("Logged in and ready! My id is: {}", bot_id.0);

    loop {
        match connection.recv_event() {
            Ok(Event::MessageCreate(message)) => {
                if let Result::Ok(Channel::Public(_)) = discord.get_channel(message.channel_id) {
                    println!("message sent with content: {}", message.content);
                    let request = protocol::parse_request(&message, &bot_id);
                    println!("Understood request : {:?}", request);
                    let responses = protocol::process_request(&mut conn, &request);
                    println!("Replying with {:?}", responses);
                    protocol::communicate_responses(
                        &mut conn,
                        &discord,
                        message.channel_id,
                        &responses,
                    );
                }
            }
            Ok(Event::ReactionAdd(reaction)) => {
                if reaction.user_id.0 != bot_id.0 {
                    match &reaction.emoji {
                        ReactionEmoji::Custom { .. } => {}
                        ReactionEmoji::Unicode(u) => {
                            // println!("Unicode reaction added: {}", u);
                            for (i, emoji) in COLUMN_EMOJI.iter().enumerate() {
                                if u == emoji {
                                    let responses = protocol::process_request(
                                        &mut conn,
                                        &Request::RespondToInteraction(
                                            reaction.user_id,
                                            reaction.message_id,
                                            i as u8,
                                        ),
                                    );
                                    protocol::communicate_responses(
                                        &mut conn,
                                        &discord,
                                        reaction.channel_id,
                                        &responses,
                                    );
                                }
                            }
                        }
                    }
                }
            }
            Ok(_) => {}
            Err(discord::Error::Closed(code, body)) => {
                println!("Gateway closed on us with code {:?}: {}", code, body);
                break;
            }
            Err(err) => println!("Receive error: {:?}", err),
        }
    }
}
