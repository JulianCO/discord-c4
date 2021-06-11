use crate::connect4::board::{Board, GameResult, GameStatus, Player};
use crate::connect4::monte_carlo_ai;
use crate::connect4::persistency;
use crate::connect4::persistency::{Error, NotCompletedReason, OngoingMatch};

use discord::model::{ChannelId, Event, Message, MessageId, ReactionEmoji, UserId};
use discord::Discord;

use rusqlite::Connection;

use rand;

use std::str::FromStr;

pub const COLUMN_EMOJI: [&str; 7] = ["1️⃣", "2️⃣", "3️⃣", "4️⃣", "5️⃣", "6️⃣", "7️⃣"];

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct MatchId(pub u64);

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
pub enum Request {
    Ignore,
    Help(HelpTopic),
    Challenge(ChannelId, UserId, UserId, PlayOrder),
    ChallengeBot(ChannelId, UserId, u8, PlayOrder),
    PlayMove(ChannelId, UserId, u8),
    RespondToInteraction(UserId, MessageId, u8),
    SeeGame(ChannelId, UserId),
    Resign(ChannelId, UserId),
}

#[derive(Debug)]
enum UserError {
    PlayerAlreadyPlaying,
    PlayerNotPlaying,
    NotYourTurn,
    IllegalMove,
}

#[derive(Debug)]
enum GameOverReason {
    PlayerWon,
    Tie,
    Resignation,
}

#[derive(Debug)]
pub enum Response {
    ShowGame(OngoingMatch, bool, Option<u8>),
    ShowHelp(HelpTopic),
    ShowError(UserId, UserError),
    BotPlaysMove(MatchId),
}

pub fn parse_request(message: &Message, bot_id: &UserId) -> Request {
    if !message.content.starts_with("!c4 ") {
        Request::Ignore
    } else {
        if message.content.starts_with("!c4 challenge") {
            if let Some(other_player) = message.mentions.get(0) {
                if other_player.id == *bot_id {
                    Request::ChallengeBot(
                        message.channel_id,
                        message.author.id,
                        5,
                        PlayOrder::Random,
                    )
                } else {
                    Request::Challenge(
                        message.channel_id,
                        message.author.id,
                        other_player.id,
                        PlayOrder::GoFirst,
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

pub fn process_request(conn: &mut Connection, request: &Request) -> Vec<Response> {
    match request {
        Request::Ignore => {
            vec![]
        }
        Request::Help(help_topic) => {
            vec![Response::ShowHelp(*help_topic)]
        }
        Request::Challenge(channel, challenger, challenged, play_order) => challenge_human(
            conn,
            channel,
            challenger,
            challenged,
            decide_random_order(*play_order),
        ),
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
                    vec![Response::ShowError(*player_id, UserError::PlayerNotPlaying)]
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
        Request::RespondToInteraction(player_id, message_id, move_no) => {
            let found_interaction =
                persistency::search_interaction(&conn, message_id.0, player_id.0);
            match found_interaction {
                Err(_) => vec![],
                Ok(ongoing_match) => process_request(
                    conn,
                    &Request::PlayMove(
                        ChannelId(ongoing_match.get_server_id()),
                        *player_id,
                        *move_no,
                    ),
                ),
            }
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
    mut human_match: persistency::HumanMatch,
    player_id: UserId,
    move_no: u8,
) -> Vec<Response> {
    let turn_ok = check_player_turn_vs_human(&human_match, player_id);

    if !turn_ok {
        vec![Response::ShowError(player_id, UserError::NotYourTurn)]
    } else if !human_match.board.is_move_legal(move_no) {
        vec![Response::ShowError(player_id, UserError::IllegalMove)]
    } else {
        human_match.board.play_move(move_no);

        let setup_interaction = match human_match.board.game_status() {
            GameStatus::GameOver(_) => {
                persistency::delete_match(conn, human_match.match_id);
                false
            }
            GameStatus::Turn(_) => {
                persistency::update_match_board(conn, human_match.match_id, &human_match.board)
                    .expect("Error updating game state");
                true
            }
        };

        vec![Response::ShowGame(
            OngoingMatch::HumanMatch(human_match),
            setup_interaction,
            Some(move_no)
        )]
    }
}

fn process_move_vs_computer(
    conn: &Connection,
    mut computer_match: persistency::ComputerMatch,
    player_id: UserId,
    move_no: u8,
) -> Vec<Response> {
    let turn_ok = check_player_turn_vs_bot(&computer_match, player_id);

    if !turn_ok {
        vec![Response::ShowError(player_id, UserError::NotYourTurn)]
    } else if !computer_match.board.is_move_legal(move_no) {
        vec![Response::ShowError(player_id, UserError::IllegalMove)]
    } else {
        computer_match.board.play_move(move_no);
        
        persistency::update_match_board(conn, computer_match.match_id, &computer_match.board);

        let mut bot_responses = match computer_match.board.game_status() {
            GameStatus::GameOver(_) => {
                persistency::delete_match(conn, computer_match.match_id);
                vec![]
            }
            GameStatus::Turn(_) => vec![Response::BotPlaysMove(MatchId(computer_match.match_id))],
        };

        let mut responses = vec![Response::ShowGame(
            OngoingMatch::ComputerMatch(computer_match),
            false,
            Some(move_no)
        )];
        responses.append(&mut bot_responses);

        responses
    }
}

fn challenge_human(
    conn: &mut Connection,
    channel: &ChannelId,
    challenger: &UserId,
    challenged: &UserId,
    play_order: PlayOrder,
) -> Vec<Response> {
    let mut red_player_id;
    let mut blue_player_id;
    let decided_play_order = decide_random_order(play_order);

    match decided_play_order {
        PlayOrder::GoFirst => {
            red_player_id = challenger.0;
            blue_player_id = challenged.0;
        }
        PlayOrder::GoSecond => {
            red_player_id = challenger.0;
            blue_player_id = challenged.0;
        }
        _ => {
            panic!("The impossible has happened");
        }
    }
    let match_id_result =
        persistency::new_human_match(conn, channel.0, red_player_id, blue_player_id);
    match match_id_result {
        Err(Error::NotCompleted(NotCompletedReason::RedAlreadyPlaying)) => {
            vec![Response::ShowError(
                UserId(red_player_id),
                UserError::PlayerAlreadyPlaying,
            )]
        }
        Err(Error::NotCompleted(NotCompletedReason::BlueAlreadyPlaying)) => {
            vec![Response::ShowError(
                UserId(blue_player_id),
                UserError::PlayerAlreadyPlaying,
            )]
        }
        Err(unknown_error) => Err(unknown_error).expect("unknown error encountered"),
        Ok(human_match) => {
            vec![
                Response::ShowGame(OngoingMatch::HumanMatch(human_match), true, None),
            ]
        }
    }
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
            vec![Response::ShowError(
                *player_id,
                UserError::PlayerAlreadyPlaying,
            )]
        }
        Err(unknown_error) => Err(unknown_error).expect("unknown error encountered"),
        Ok(computer_match) => {
            vec![
                Response::ShowGame(OngoingMatch::ComputerMatch(computer_match), true, None),
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
            vec![Response::ShowError(
                *player_id,
                UserError::PlayerAlreadyPlaying,
            )]
        }

        Err(unknown_error) => Err(unknown_error).expect("unknown error encountered"),

        Ok(initial_bot_match) => {    
            let match_id = initial_bot_match.match_id;
            vec![
                Response::ShowGame(OngoingMatch::ComputerMatch(initial_bot_match), true, None),
                Response::BotPlaysMove(MatchId(match_id)),
            ]
        }
    }
}

fn play_bot_move(conn: &Connection, match_id: MatchId) -> Vec<Response> {
    let mut match_new = persistency::retrieve_match_by_id(conn, match_id.0).expect("bot tried to play move in match with invalid id");

    let mut bot_match_new =
        match match_new {
            OngoingMatch::HumanMatch(_) => panic!("bot tried to play move in human match"),
            OngoingMatch::ComputerMatch(c) => c,
        };
    
    let suggested_move = monte_carlo_ai::ai_move(
        &bot_match_new.board,
        search_depth_at_level(bot_match_new.ai_level),
    )
    .expect("AI failure :(");
    bot_match_new.board.play_move(suggested_move);

    let responses = match bot_match_new.board.game_status() {
        GameStatus::GameOver(_) => {
            persistency::delete_match(conn, match_id.0);
            vec![
                Response::ShowGame(OngoingMatch::ComputerMatch(bot_match_new), false, Some(suggested_move))
            ]
        }
        GameStatus::Turn(_) => {
            persistency::update_match_board(conn, bot_match_new.match_id, &bot_match_new.board)
                .expect("DB error when updating match");
            vec![
                Response::ShowGame(OngoingMatch::ComputerMatch(bot_match_new), true, Some(suggested_move)),
            ]
        }
    };

    responses
}

pub fn communicate_responses(
    conn: &mut Connection,
    discord: &Discord,
    channel_id: ChannelId,
    responses: &Vec<Response>,
) {
    for r in responses {
        communicate_response(conn, discord, channel_id, r);
    }
}

pub fn communicate_response(
    conn: &mut Connection,
    discord: &Discord,
    channel_id: ChannelId,
    response: &Response,
) {
    match response {
        Response::ShowGame(ongoing_match, prompt_player, _last_move) => show_game(conn, discord, channel_id, ongoing_match, *prompt_player),
        Response::ShowHelp(_help_topic) => show_help(conn, discord, channel_id),
        Response::ShowError(user_id, user_error) => show_error(conn, discord, channel_id, user_id, user_error),
        Response::BotPlaysMove(match_id) => {
            communicate_responses(conn, discord, channel_id, &play_bot_move(conn, *match_id));
        },
    };
}

fn show_game(
    conn: &mut Connection,
    discord: &Discord,
    channel_id: ChannelId,
    ongoing_match : &OngoingMatch, 
    prompt_player: bool,
) {
    let response_string = {
        let board = match ongoing_match {
            OngoingMatch::HumanMatch(h) => &h.board,
            OngoingMatch::ComputerMatch(c) => &c.board,
        };

        let mut board_string = board.display(
            ":red_circle:",
            ":blue_circle:",
            ":white_circle:",
            "",
            "",
            "\n",
            "",
        );

        board_string.push_str(":one::two::three::four::five::six::seven:");

        board_string
    };
    
    let message = 
        match ongoing_match.get_message_id() {
            Some(message_id) => discord
                .edit_message(channel_id, MessageId(message_id), &response_string)
                .expect("failed to edit message"),
            None => discord
                .send_message(channel_id, &response_string, "", false)
                .expect("failed to send message"),
        };
    
    if prompt_player {
        for (i, emoji) in COLUMN_EMOJI.iter().enumerate() {
            if ongoing_match.get_board().is_move_legal(i as u8) {
                println!("attempting to react with {}", emoji);
                match discord.add_reaction(
                    channel_id,
                    message.id,
                    ReactionEmoji::Unicode(emoji.to_string()),
                ) {
                    Err(e) => println!("got error: {:?}", e),
                    _ => {}
                };
            }
        }
        persistency::register_interaction(conn, message.id.0, &ongoing_match);
    }
}

fn show_help(
    conn: &mut Connection,
    discord: &Discord,
    channel_id: ChannelId,
) {
    discord
        .send_message(channel_id, "Help message", "", false)
        .expect("failed to send message");
}


fn show_error(
    conn: &mut Connection,
    discord: &Discord,
    channel_id: ChannelId,
    user_id : &UserId, 
    user_error: &UserError, 
) {
    discord
        .send_message(channel_id, "Help message", "", false)
        .expect("failed to send message");
}

fn check_player_turn_vs_bot(
    computer_match: &persistency::ComputerMatch,
    player_id: UserId,
) -> bool {
    match computer_match.board.game_status() {
        GameStatus::Turn(Player::Red) => computer_match.player_is_red,
        GameStatus::Turn(Player::Blue) => !computer_match.player_is_red,
        _ => false,
    }
}

fn check_player_turn_vs_human(human_match: &persistency::HumanMatch, player_id: UserId) -> bool {
    match human_match.board.game_status() {
        GameStatus::Turn(Player::Red) => player_id.0 == human_match.red_player_id,
        GameStatus::Turn(Player::Blue) => player_id.0 == human_match.blue_player_id,
        _ => false,
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

