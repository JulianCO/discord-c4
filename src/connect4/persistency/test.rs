use super::*;
use std::fs;
use crate::connect4::board;
use crate::connect4::board::{Player, GameResult};

#[test]
fn game_creation_stops_duplicates() {
    let _ = fs::remove_file("test1.sqlite");
    let mut conn = initialize("test1.sqlite").expect("Failed to create database");
    
    new_human_match(&mut conn, 1, 12, 21).expect("failed to create a game");
    
    let error_1 = new_human_match(&mut conn, 1, 12, 22).err().expect("Creation should have failed");
    assert_eq!(error_1, Error::NotCompleted(NotCompletedReason::RedAlreadyPlaying));
    
    let error_2 = new_human_match(&mut conn, 1, 22, 12).err().expect("Creation should have failed");
    assert_eq!(error_2, Error::NotCompleted(NotCompletedReason::BlueAlreadyPlaying));
    
    let error_3 = new_human_match(&mut conn, 1, 21, 22).err().expect("Creation should have failed");
    assert_eq!(error_3, Error::NotCompleted(NotCompletedReason::RedAlreadyPlaying));
    
    let error_4 = new_human_match(&mut conn, 1, 22, 21).err().expect("Creation should have failed");
    assert_eq!(error_4, Error::NotCompleted(NotCompletedReason::BlueAlreadyPlaying));
    
    let error_5 = new_computer_match(&mut conn, 1, 21, false, 5).err().expect("Creation should have failed");
    assert_eq!(error_5, Error::NotCompleted(NotCompletedReason::PlayerAlreadyPlaying));
    
    drop(conn);
    fs::remove_file("test1.sqlite").expect("failed to remove temp database");
}

#[test]
fn retrieving_empty_game_works() {
    let _ = fs::remove_file("test2.sqlite");
    let mut conn = initialize("test2.sqlite").expect("Failed to create database");
    
    let e = Board::empty_board(); 
    
    new_human_match(&mut conn, 1, 12, 21).expect("failed to create a game");
    new_computer_match(&mut conn, 1, 13, true, 5).expect("failed to create a game");
    
    let found_human_match = retrieve_match_by_player(&conn, 1, 21).expect("Match just created not found");
    let found_computer_match = retrieve_match_by_player(&conn, 1, 13).expect("Match just created not found");
    
    match found_human_match {
        OngoingMatch::ComputerMatch(_) => 
            panic!("Found computer match where a human match was inserted"),
        OngoingMatch::HumanMatch(m) => 
            assert_eq!(m.board.serialize(), e.serialize())
    }
    
    match found_computer_match {
        OngoingMatch::HumanMatch(_) => 
            panic!("Found human match where a computer match was inserted"),
        OngoingMatch::ComputerMatch(m) => 
            assert_eq!(m.board.serialize(), e.serialize())
    }
    
    drop(conn);
    fs::remove_file("test2.sqlite").expect("failed to remove temp database");
}

fn example_match_in_database(conn: &mut Connection, p1 : u64, p2 : u64,moves : &[u8], expected_result : board::GameResult) {
    let mut e = Board::empty_board();
    let mut m : OngoingMatch;
    let mut m_id = 444444u64;
    
    new_human_match(conn, 1, p1, p2).expect("Failed to create a game");
    
    for k in moves {
        m = retrieve_match_by_player(conn, 1, p2).expect("Failed to retrieve game");
        e = match m {
            OngoingMatch::ComputerMatch(_) =>
                panic!("Found computer match where a human match was inserted"),
            OngoingMatch::HumanMatch(h) => {
                m_id = h.match_id;
                h.board
            }
        };
        assert!(e.is_move_legal(*k));
        e.play_move(*k);
        
        update_match_board(conn, m_id, &e);
    }
    assert!(e.game_status() == board::GameStatus::GameOver(expected_result));
}

#[test]
fn test_sequencial_games_in_database() {
    let _ = fs::remove_file("test3.sqlite");
    let mut conn = initialize("test3.sqlite").expect("Failed to create database");
    
    let game1 = vec![3, 3, 2, 1, 3, 5, 2, 2, 4, 3, 1, 2, 5, 1, 1, 3, 5, 5, 5, 2, 1, 2];
    let game2 = vec![3, 3, 3, 5, 5, 2, 3, 3, 2, 2, 2, 2, 6, 1, 4, 6, 6, 6, 1, 3, 1, 0, 1, 1, 4, 6, 4];
    let game3 = vec![2, 3, 3, 3, 2, 2, 5, 5, 3, 5, 5, 2, 2, 6, 6, 3, 3, 5, 4, 0, 0, 0, 5, 6, 1, 1, 1, 6, 0, 4];
    let game4 = vec![4, 3, 3, 3, 3, 4, 5, 4, 4, 0, 3, 0, 3, 4, 2, 4, 1, 0, 0, 0, 0, 1, 1, 1, 1, 1, 2, 2, 2, 5, 5, 5, 6, 6, 6, 5, 6, 5, 2, 2, 6, 6];
    
    
    example_match_in_database(&mut conn, 12, 21, &game1, GameResult::Winner(Player::Blue));
    example_match_in_database(&mut conn, 13, 31, &game2, GameResult::Winner(Player::Red));
    example_match_in_database(&mut conn, 14, 41, &game3, GameResult::Winner(Player::Blue));
    example_match_in_database(&mut conn, 15, 51, &game4, GameResult::Tie);
    
    drop(conn);
    fs::remove_file("test3.sqlite").expect("failed to remove temp database");
}

#[test]
fn create_interact_delete() {
    let _ = fs::remove_file("test4.sqlite");
    let mut conn = initialize("test4.sqlite").expect("Failed to create database");
    
    let server_id = 3;
    let red_player_id = 22;
    let blue_player_id = 33;
    
    let match_id1 = new_human_match(&mut conn, server_id, red_player_id, blue_player_id).expect("failed to create match");
    
    let mut ongoing_match = retrieve_match_by_player(&conn, server_id, blue_player_id).expect("failed to retrieve game");
    
    let message_id = 12;
    
    register_interaction(&conn, message_id, &ongoing_match).expect("failed to register interaction");
    
    let error1 = search_interaction(&conn, message_id, blue_player_id).err().expect("No interaction should have been found");
    assert_eq!(error1, Error::NotCompleted(NotCompletedReason::NoSuchInteraction));
    
    search_interaction(&conn, message_id, red_player_id).expect("Interaction not found");
    
    let mut board = 
        match &mut ongoing_match {
            OngoingMatch::ComputerMatch(_) =>
                panic!("Match should be a human match"),
            OngoingMatch::HumanMatch(h) =>
                &mut h.board
        };
    
    board.play_move(2);
    
    register_interaction(&conn, message_id, &ongoing_match).expect("failed to register interaction");
    
    let error2 = search_interaction(&conn, message_id, red_player_id).err().expect("No interaction should have been found");
    assert_eq!(error2, Error::NotCompleted(NotCompletedReason::NoSuchInteraction));
    
    search_interaction(&conn, message_id, blue_player_id).expect("Interaction not found");
    
    drop(conn);
    fs::remove_file("test4.sqlite").expect("failed to remove temp database");
}
