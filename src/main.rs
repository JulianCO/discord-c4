mod connect4;

use self::connect4::board::Board;
use self::connect4::monte_carlo_ai;
use self::connect4::persistency;
use self::connect4::persistency::{Error, NotCompletedReason, OngoingMatch};
use discord::model::{ChannelId, Event, Message, MessageId, UserId};
use discord::Discord;
use rand;
use rusqlite::Connection;
use std::env;
use std::str::FromStr;

// invite through https://discord.com/api/oauth2/authorize?client_id=805143667392118794&scope=bot&permissions=3136

#[derive(Debug, PartialEq)]
struct MatchId(pub u64);

#[derive(Debug, PartialEq, Clone, Copy)]
enum HelpTopic {
    General,
    Challenge,
    Play,
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum PlayOrder {
    GoFirst,
    GoSecond,
    Random,
}

#[derive(Debug, PartialEq)]
enum Request {
    Ignore,
    Help(HelpTopic),
    Challenge(ChannelId, UserId, UserId, PlayOrder),
    ChallengeBot(ChannelId, UserId, u8, PlayOrder),
    PlayMove(ChannelId, UserId, u8),
    RespondToInteraction(UserId, MessageId, u8),
    SeeGame(ChannelId, UserId),
    Resign(ChannelId, UserId),
}

enum UserError {
    PlayerAlreadyPlaying(UserId),
    PlayerNotPlaying(UserId),
    NotYourTurn(UserId),
}

enum GameOverReason {
    PlayerWon,
    Tie,
    Resignation,
}

enum Response {
    ShowGame(OngoingMatch),
    ShowHelp(HelpTopic),
    ShowChallengeMessage(UserId, UserId),
    ShowBotChallengeMessage(UserId),
    AnnounceMove(u8),
    ShowHumanMatchOver(UserId, UserId, GameOverReason),
    ShowComputerMatchOver(bool, GameOverReason),
    ShowError(UserError),
}

fn main() {
    let discord = Discord::from_bot_token(
        &env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not set in environment"),
    )
    .expect("login failed");

    let mut conn =
        persistency::initialize("test_env.sqlite").expect("failed to initialize database");

    let (mut connection, _) = discord.connect().expect("connect failed");
    let bot_id = discord.get_current_user().expect("failed to find self").id;
    println!("Logged in and ready! My id is: {}", bot_id.0);

    loop {
        match connection.recv_event() {
            Ok(Event::MessageCreate(message)) => {
                println!("message sent with content: {}", message.content);
                let request = parse_request(&message, &bot_id);
                println!("Understood request : {:?}", request);
                let responses = process_request(&mut conn, &request);
                communicate_responses(&mut conn, &discord, message.channel_id, &responses);
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

fn parse_request(message: &Message, bot_id: &UserId) -> Request {
    if !message.content.starts_with("!c4 ") {
        Request::Ignore
    } else {
        if message.content.starts_with("!c4 challenge") {
            if let Some(other_player) = message.mentions.get(0) {
                if other_player.id == *bot_id {
                    Request::ChallengeBot(
                        message.channel_id,
                        message.author.id,
                        0,
                        PlayOrder::Random,
                    )
                } else {
                    Request::Challenge(
                        message.channel_id,
                        message.author.id,
                        other_player.id,
                        PlayOrder::Random,
                    )
                }
            } else {
                Request::Help(HelpTopic::Challenge)
            }
        } else if let Some(move_no) = message.content.strip_prefix("!c4 play ") {
            if let Ok(n) = u8::from_str(move_no.trim()) {
                if n > 7 || n < 1 {
                    Request::Help(HelpTopic::Play)
                } else {
                    Request::PlayMove(message.channel_id, message.author.id, n - 1)
                }
            } else {
                Request::Help(HelpTopic::Play)
            }
        } else if message.content.starts_with("!c4 resign") {
            Request::Resign(message.channel_id, message.author.id)
        } else if message.content.starts_with("!c4 see") {
            Request::SeeGame(message.channel_id, message.author.id)
        } else {
            Request::Help(HelpTopic::General)
        }
    }
}

fn process_request(conn: &mut Connection, request: &Request) -> Vec<Response> {
    match request {
        Request::Ignore => {
            vec![]
        }
        Request::Help(help_topic) => {
            vec![Response::ShowHelp(*help_topic)]
        }
        Request::Challenge(_channel, _challenger, _challenged, _play_order) => {
            vec![]
        }
        Request::ChallengeBot(channel_id, player_id, ai_level, play_order) => {
            match decide_random_order(*play_order) {
                PlayOrder::GoFirst => {
                    challenge_bot_go_first(conn, channel_id, player_id, *ai_level)
                }
                PlayOrder::GoSecond => {
                    challenge_bot_go_second(conn, channel_id, player_id, *ai_level)
                }
                PlayOrder::Random => panic!("The impossible has happened"),
            }
        }
        Request::PlayMove(channel_id, player_id, move_no) => {
            let found_match =
                persistency::retrieve_match_by_player(conn, channel_id.0, player_id.0);
            match found_match {
                Err(Error::NotCompleted(NotCompletedReason::PlayerHasNoMatches)) => {
                    vec![Response::ShowError(UserError::PlayerNotPlaying(*player_id))]
                }

                Err(_) => {
                    panic!("Unknown error retrieving match")
                }
                Ok(OngoingMatch::HumanMatch(human_match)) => {
                    process_move_vs_human(conn, human_match, *player_id, *move_no)
                }

                Ok(OngoingMatch::ComputerMatch(computer_match)) => {
                    process_move_vs_computer(conn, computer_match, *player_id, *move_no)
                }
            }
        }
        Request::RespondToInteraction(_player_id, _message_id, _move_no) => {
            vec![]
        }
        Request::SeeGame(_channel, _player_id) => {
            vec![]
        }
        Request::Resign(_channel, _player_id) => {
            vec![]
        }
    }
}

fn process_move_vs_human(
    conn: &Connection,
    human_match: persistency::HumanMatch,
    player_id: UserId,
    move_no: u8,
) -> Vec<Response> {
    vec![]
}

fn process_move_vs_computer(
    conn: &Connection,
    computer_match: persistency::ComputerMatch,
    player_id: UserId,
    move_no: u8,
) -> Vec<Response> {
    vec![]
}

fn challenge_bot_go_first(
    conn: &mut Connection,
    channel_id: &ChannelId,
    player_id: &UserId,
    ai_level: u8,
) -> Vec<Response> {
    let match_id_result =
        persistency::new_computer_match(conn, channel_id.0, player_id.0, true, ai_level);
    match match_id_result {
        Err(Error::NotCompleted(NotCompletedReason::PlayerAlreadyPlaying)) => {
            vec![Response::ShowError(UserError::PlayerAlreadyPlaying(
                *player_id,
            ))]
        }
        Err(unknown_error) => Err(unknown_error).expect("unknown error encountered"),
        Ok(computer_match) => {
            vec![
                Response::ShowBotChallengeMessage(*player_id),
                Response::ShowGame(OngoingMatch::ComputerMatch(computer_match)),
            ]
        }
    }
}

fn challenge_bot_go_second(
    conn: &mut Connection,
    channel_id: &ChannelId,
    player_id: &UserId,
    ai_level: u8,
) -> Vec<Response> {
    let match_id_result =
        persistency::new_computer_match(conn, channel_id.0, player_id.0, false, ai_level);
    match match_id_result {
        Err(Error::NotCompleted(NotCompletedReason::PlayerAlreadyPlaying)) => {
            vec![Response::ShowError(UserError::PlayerAlreadyPlaying(
                *player_id,
            ))]
        }

        Err(unknown_error) => Err(unknown_error).expect("unknown error encountered"),

        Ok(initial_bot_match) => {
            let mut bot_responses = play_bot_move(conn, &initial_bot_match);

            let mut responses = vec![
                Response::ShowBotChallengeMessage(*player_id),
                Response::ShowGame(OngoingMatch::ComputerMatch(initial_bot_match)),
            ];

            responses.append(&mut bot_responses);
            responses
        }
    }
}

fn play_bot_move(conn: &Connection, bot_match: &persistency::ComputerMatch) -> Vec<Response> {
    let mut bot_match_new = bot_match.clone();

    let suggested_move = monte_carlo_ai::ai_move(
        &bot_match_new.board,
        search_depth_at_level(bot_match_new.ai_level),
    )
    .expect("AI failure :(");
    bot_match_new.board.play_move(suggested_move);

    persistency::update_match_board(conn, bot_match_new.match_id, &bot_match_new.board)
        .expect("DB error when updating match");

    vec![
        Response::AnnounceMove(suggested_move),
        Response::ShowGame(OngoingMatch::ComputerMatch(bot_match_new)),
    ]
}

fn communicate_responses(
    conn: &mut Connection,
    discord: &Discord,
    channel_id: ChannelId,
    responses: &Vec<Response>,
) -> persistency::Result<()> {
    for r in responses {
        communicate_response(conn, discord, channel_id, r)?;
    }
    Ok(())
}

fn communicate_response(
    conn: &mut Connection,
    discord: &Discord,
    channel_id: ChannelId,
    response: &Response,
) -> persistency::Result<()> {
    match response {
        Response::ShowGame(_ongoing_match) => Ok(()),
        Response::ShowHelp(_help_topic) => Ok(()),
        Response::ShowChallengeMessage(_challenger, _challenged) => Ok(()),
        Response::ShowBotChallengeMessage(_challenger) => Ok(()),
        Response::AnnounceMove(_ai_move) => Ok(()),
        Response::ShowHumanMatchOver(_winner, _loser, _game_over_reason) => Ok(()),
        Response::ShowComputerMatchOver(_win, _game_over_reason) => Ok(()),
        Response::ShowError(_user_error) => Ok(()),
    }
}

fn decide_random_order(play_order: PlayOrder) -> PlayOrder {
    match play_order {
        PlayOrder::Random => {
            if rand::random() {
                PlayOrder::GoFirst
            } else {
                PlayOrder::GoSecond
            }
        }
        definite_order => definite_order,
    }
}

fn search_depth_at_level(ai_level: u8) -> u32 {
    32768
}

fn old_main() {
    let mut b = Board::empty_board();
    b.play_move(3);
    b.play_move(2);
    b.play_move(3);
    b.play_move(3);
    b.play_move(3);
    b.play_move(4);
    b.play_move(4);

    let example_board = b.display("r", "b", " ", "|", "|", "|\n", "+-+-+-+-+-+-+-+\n");
    let example_discord = b.display(
        ":red_circle:",
        ":blue_circle:",
        ":white_circle:",
        "",
        "",
        "\n",
        "",
    );

    print!("{}", example_board);
    println!("");
    print!("{}", example_discord);

    let suggested_move = monte_carlo_ai::ai_move(&b, 32768);

    match suggested_move {
        Ok(x) => println!("The ai suggest playing the {} column", x + 1),
        Err(_) => println!("The AI failed!"),
    };
}
