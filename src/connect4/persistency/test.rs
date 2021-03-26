use super::*;
use std::fs;

#[test]
fn game_creation_stops_duplicates() {
    let mut conn = initialize("test1.sqlite").expect("Failed to create database");
    
    new_human_match(&mut conn, 1, 12, 21).expect("failed to create a game");
    
    let error_1 = new_human_match(&mut conn, 1, 12, 22).err().expect("Creation should have failed");
    assert_eq!(error_1, Error::NotCompleted(NotCompletedReason::RedAlreadyPlaying));
    
    let error_2 = new_human_match(&mut conn, 1, 23, 12).err().expect("Creation should have failed");
    assert_eq!(error_2, Error::NotCompleted(NotCompletedReason::BlueAlreadyPlaying));
    
    let error_3 = new_human_match(&mut conn, 1, 21, 24).err().expect("Creation should have failed");
    assert_eq!(error_3, Error::NotCompleted(NotCompletedReason::RedAlreadyPlaying));
    
    let error_4 = new_human_match(&mut conn, 1, 25, 21).err().expect("Creation should have failed");
    assert_eq!(error_4, Error::NotCompleted(NotCompletedReason::BlueAlreadyPlaying));
    
    drop(conn);
    fs::remove_file("test1.sqlite").expect("failed to remove temp database");
}


