use rusqlite::{Connection};
use rusqlite::params;
use rusqlite::OptionalExtension;
use std::string::String;

use crate::connect4::board::{Board, GameStatus, Player};

#[derive(Debug, PartialEq, Eq)]
pub enum NotCompletedReason {
    RedAlreadyPlaying,
    BlueAlreadyPlaying,
    PlayerAlreadyPlaying,
    PlayerHasNoMatches,
    UnrecoverableError,
    InvalidAiLevel,
    InteractionRequestedForGameOver,
    NoSuchInteraction
}

#[derive(Debug, PartialEq)]
pub enum Error {
    SqliteError(rusqlite::Error),
    NotCompleted(NotCompletedReason)
}

#[derive(Debug, PartialEq, Eq)]
pub struct HumanMatch { 
    pub match_id : u64,
    pub server_id : u64,
    pub red_player_id : u64,
    pub blue_player_id : u64,
    pub board : Board
}

#[derive(Debug, PartialEq, Eq)]
pub struct ComputerMatch { 
    pub match_id : u64,
    pub server_id : u64,
    pub player_id : u64,
    pub player_is_red : bool,
    pub ai_level : u8,
    pub board : Board
}

#[derive(Debug, PartialEq, Eq)]
pub enum OngoingMatch {
    HumanMatch(HumanMatch),
    ComputerMatch(ComputerMatch)
}

pub struct PendingInteraction {
    interaction_id : u64,
    match_id : u64,
    message_id : u64,
    player_id : u64
}

struct DatabaseRow {
    match_id : i64,
    server_id : i64,
    red_player_id : i64,
    blue_player_id : i64,
    red_pieces : i64,
    blue_pieces : i64
}

impl From<rusqlite::Error> for Error {
    fn from(e : rusqlite::Error) -> Error {
        Error::SqliteError(e)
    }
}

impl OngoingMatch {
    pub fn get_id(&self) -> u64 {
        match self {
            OngoingMatch::HumanMatch(h) =>
                h.match_id,
            OngoingMatch::ComputerMatch(c) => 
                c.match_id
        }
    }
}

pub type Result<T, E = Error> = core::result::Result<T, E>;

pub fn initialize(db_name : &str) -> Result<Connection> {
    let conn =  Connection::open(db_name)?;
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS matches (
            match_id INTEGER PRIMARY KEY,
            server_id INTEGER NOT NULL,
            red_player_id INTEGER NOT NULL,
            blue_player_id INTEGER NOT NULL,
            red_pieces INTEGER NOT NULL,
            blue_pieces INTEGER NOT NULL
            );", params![])?;
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS interactions (
            interaction_id INTEGER PRIMARY KEY,
            message_id INTEGER NOT NULL,
            match_id INTEGER NOT NULL,
            prompted_player_id INTEGER NOT NULL
            );", params![])?;
    
    Ok(conn)
}

pub fn new_human_match(conn : &mut Connection, server_id : u64, red_id : u64, blue_id : u64) -> Result<HumanMatch> {
    let tx = conn.transaction()?;
    
    let e = Board::empty_board();
    let (red_pieces, blue_pieces) = e.serialize();
    
    tx.execute(
        "INSERT INTO matches
            (server_id, red_player_id, blue_player_id, red_pieces, blue_pieces)
            VALUES
            (?1, ?2, ?3, ?4, ?5)
            ;", 
        params![server_id as i64, red_id as i64, blue_id as i64, red_pieces as i64, blue_pieces as i64])?;
    
    let mut stmt1 = tx.prepare(
        "SELECT match_id 
            FROM matches 
            WHERE server_id=?1
            AND (red_player_id=?2 OR blue_player_id=?2)
            ;")?;
    
    let mut red_found = 
        stmt1.query_map(
            params![server_id as i64, red_id as i64], 
            |row| { row.get(0) }
            )?;
    
    let match_id : i64 = 
        match red_found.next() {
            Some(Ok(n)) => 
                match red_found.next() {
                    None =>
                        Ok(n),
                    Some(_) =>
                        Err(Error::NotCompleted(NotCompletedReason::RedAlreadyPlaying))
                },
            Some(Err(r)) =>
                Err(Error::SqliteError(r)),
            None =>
                Err(Error::NotCompleted(NotCompletedReason::UnrecoverableError))
        }?;
    
    drop(red_found);
    
    let mut blue_found =
        stmt1.query_map(
            params![server_id as i64, blue_id as i64], 
            |row| { row.get(0) } 
            )?;
    
    let match_id_2 : i64 = 
        match blue_found.next() {
            Some(Ok(n)) => 
                match blue_found.next() {
                    None =>
                        Ok(n),
                    Some(_) =>
                        Err(Error::NotCompleted(NotCompletedReason::BlueAlreadyPlaying))
                },
            Some(Err(r)) =>
                Err(Error::SqliteError(r)),
            None =>
                Err(Error::NotCompleted(NotCompletedReason::UnrecoverableError))
        }?;
    
    drop(blue_found);
    drop(stmt1);
    
    if match_id == match_id_2 {
        tx.commit()?;
        Ok(
            HumanMatch {
                match_id : match_id as u64,
                server_id,
                red_player_id : red_id,
                blue_player_id : blue_id,
                board : e
            }
        )
    } else {
        Err(Error::NotCompleted(NotCompletedReason::UnrecoverableError))
    }
}

pub fn new_computer_match(conn : &mut Connection, server_id : u64, player_id : u64, player_is_red : bool, ai_level : u8) -> Result<ComputerMatch> {
    if ai_level > 10 {
        return Err(Error::NotCompleted(NotCompletedReason::InvalidAiLevel));
    }
    
    let tx = conn.transaction()?;
    
    let e = Board::empty_board();
    let (red_pieces, blue_pieces) = e.serialize();
    
    let query = 
        if player_is_red {
            format!(
                "INSERT INTO matches
                    (server_id, red_player_id, blue_player_id, red_pieces, blue_pieces)
                    VALUES
                    (?1, ?2, -{}, ?3, ?4)
                    ;", ai_level)
        }
        else {
            format!(
                "INSERT INTO matches
                    (server_id, red_player_id, blue_player_id, red_pieces, blue_pieces)
                    VALUES
                    (?1, -{}, ?2, ?3, ?4)
                    ;", ai_level)
        };
    
    tx.execute(
        &query,
        params![server_id as i64, player_id as i64, red_pieces as i64, blue_pieces as i64])?;
    
    let mut stmt1 = tx.prepare(
        "SELECT match_id 
            FROM matches 
            WHERE server_id=?1
            AND (red_player_id=?2 OR blue_player_id=?2)
            ;")?;
    
    let mut player_found = 
        stmt1.query_map(
            params![server_id as i64, player_id as i64], 
            |row| { row.get(0) }
            )?;
    
    let match_id : i64 = 
        match player_found.next() {
            Some(Ok(n)) => 
                match player_found.next() {
                    None =>
                        Ok(n),
                    Some(_) =>
                        Err(Error::NotCompleted(NotCompletedReason::PlayerAlreadyPlaying))
                },
            Some(Err(r)) =>
                Err(Error::SqliteError(r)),
            None =>
                Err(Error::NotCompleted(NotCompletedReason::UnrecoverableError))
        }?;
    
    drop(player_found);
    drop(stmt1);
    
    tx.commit()?;
    Ok(
        ComputerMatch {
            match_id : match_id as u64,
            server_id,
            player_id,
            player_is_red,
            ai_level,
            board : e
        }
    )
}

fn data_row_to_match(row : &DatabaseRow) -> OngoingMatch {
    let board = Board::unserialize((row.red_pieces as u64, row.blue_pieces as u64));
    if row.red_player_id < 0 && row.red_player_id > -10 {
        OngoingMatch::ComputerMatch( ComputerMatch {
            match_id : row.match_id as u64,
            server_id : row.server_id as u64,
            player_id : row.blue_player_id as u64,
            player_is_red : false,
            ai_level : (-row.red_player_id) as u8,
            board
        })
    }
    else if row.blue_player_id < 0 && row.blue_player_id > -10 {
        OngoingMatch::ComputerMatch( ComputerMatch {
            match_id : row.match_id as u64,
            server_id : row.server_id as u64,
            player_id : row.red_player_id as u64,
            player_is_red : true,
            ai_level : (-row.blue_player_id) as u8,
            board
        })
    } 
    else {
        OngoingMatch::HumanMatch( HumanMatch {
            match_id : row.match_id as u64,
            server_id : row.server_id as u64,
            red_player_id : row.red_player_id as u64,
            blue_player_id : row.blue_player_id as u64,
            board
        })
    }
}
            

pub fn retrieve_match_by_player(conn : &Connection, server_id : u64, player_id : u64) -> Result<OngoingMatch> {
    let corresponding_row_opt = conn.query_row(
        "SELECT match_id, red_player_id, blue_player_id, red_pieces, blue_pieces
            FROM matches 
            WHERE server_id=?1
            AND (red_player_id=?2 OR blue_player_id=?2)
            ;",
        params![server_id as i64, player_id as i64],
        |row| 
            Ok( DatabaseRow {
                match_id : row.get(0)?,
                server_id : server_id as i64,
                red_player_id : row.get(1)?,
                blue_player_id : row.get(2)?,
                red_pieces : row.get(3)?,
                blue_pieces : row.get(4)?
            })
        ).optional()?;
    match corresponding_row_opt {
        None => 
            Err(Error::NotCompleted(NotCompletedReason::PlayerHasNoMatches)),
        Some(corresponding_row) =>
            Ok(data_row_to_match(&corresponding_row))
    }
}

pub fn retrieve_match_by_id(conn : &Connection, match_id : u64) -> Result<OngoingMatch> {
    let corresponding_row_opt = conn.query_row(
        "SELECT match_id, server_id, red_player_id, blue_player_id, red_pieces, blue_pieces
            FROM matches 
            WHERE match_id = ?1
            ;",
        params![match_id as i64],
        |row| 
            Ok( DatabaseRow {
                match_id : row.get(0)?,
                server_id : row.get(1)?,
                red_player_id : row.get(2)?,
                blue_player_id : row.get(3)?,
                red_pieces : row.get(4)?,
                blue_pieces : row.get(5)?
            })
        ).optional()?;
    match corresponding_row_opt {
        None => 
            Err(Error::NotCompleted(NotCompletedReason::UnrecoverableError)),
        Some(corresponding_row) =>
            Ok(data_row_to_match(&corresponding_row))
    }
}

pub fn update_match_board(conn : &Connection, match_id : u64, board : &Board) -> Result<()> {
    conn.execute(
        "DELETE FROM interactions
            WHERE match_id = ?1;",
        params![match_id as i64])?;

    let (red_pieces, blue_pieces) = board.serialize();
    conn.execute(
        "UPDATE matches
            SET red_pieces = ?1, blue_pieces = ?2
            WHERE match_id = ?3",
        params![red_pieces as i64, blue_pieces as i64, match_id as i64])?;
    Ok(())
}

pub fn delete_match(conn : &Connection, match_id : u64) -> Result<()> {
    conn.execute(
        "DELETE FROM matches
            WHERE match_id = ?1;",
        params![match_id as i64])?;
        
    conn.execute(
        "DELETE FROM interactions
            WHERE match_id = ?1;",
        params![match_id as i64])?;
    
    Ok(())
}

pub fn search_interaction(conn : &Connection, message_id : u64, player_id : u64) -> Result<OngoingMatch> {
    let found_match_opt : Option<i64> = conn.query_row(
        "SELECT match_id FROM interactions
            WHERE message_id = ?1 AND prompted_player_id = ?2;",
        params![message_id as i64, player_id as i64],
        |row| row.get(0)).optional()?;
    
    match found_match_opt {
        None =>
            Err(Error::NotCompleted(NotCompletedReason::NoSuchInteraction)),
        Some(found_match) =>
            retrieve_match_by_id(conn, found_match as u64)
    }
}
            

pub fn register_interaction(conn : &Connection, message_id : u64, ongoing_match : &OngoingMatch) -> Result<()> {
    conn.execute(
        "DELETE FROM interactions
            WHERE match_id = ?1;",
        params![ongoing_match.get_id() as i64])?;
    
    let player_id = 
        match ongoing_match {
            OngoingMatch::HumanMatch(h) =>
                match h.board.game_status() {
                    GameStatus::Turn(t) =>
                        if t == Player::Red {
                            Ok(h.red_player_id)
                        }
                        else {
                            Ok(h.blue_player_id)
                        },
                    GameStatus::GameOver(_) =>
                        Err(Error::NotCompleted(NotCompletedReason::InteractionRequestedForGameOver))
                },
            OngoingMatch::ComputerMatch(c) =>
                match c.board.game_status() {
                    GameStatus::Turn(_) => 
                        Ok(c.player_id),
                    GameStatus::GameOver(_) =>
                        Err(Error::NotCompleted(NotCompletedReason::InteractionRequestedForGameOver))
                }
        }?;
                
    conn.execute(
        "INSERT INTO interactions (message_id, match_id, prompted_player_id)
            VALUES (?1, ?2, ?3);",
        params![message_id as i64, ongoing_match.get_id() as i64, player_id as i64])?;
    
    Ok(())
}
        

#[cfg(test)]
mod test;
    

