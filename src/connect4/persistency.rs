use rusqlite::{Connection};
use rusqlite::params;

use crate::connect4::board::Board;

#[derive(Debug, PartialEq, Eq)]
pub enum NotCompletedReason {
    RedAlreadyPlaying,
    BlueAlreadyPlaying,
    UnrecoverableError
}

#[derive(Debug, PartialEq)]
pub enum Error {
    SqliteError(rusqlite::Error),
    NotCompleted(NotCompletedReason)
}

impl From<rusqlite::Error> for Error {
    fn from(e : rusqlite::Error) -> Error {
        Error::SqliteError(e)
    }
}

type Result<T, E = Error> = core::result::Result<T, E>;

pub fn initialize(db_name : &str) -> Result<Connection> {
    let mut conn =  Connection::open(db_name)?;
    
    conn.execute(
        "CREATE TABLE IF NOT EXISTS matches (
            match_id INTEGER PRIMARY KEY,
            server_id INTEGER NOT NULL,
            red_player_id INTEGER,
            blue_player_id INTEGER,
            red_pieces INTEGER,
            blue_pieces INTEGER
            );", params![])?;
    Ok(conn)
}

pub fn new_human_match(conn : &mut Connection, server_id : u64, red_id : u64, blue_id : u64) -> Result<u64> {
    let tx = conn.transaction()?;
    
    let e = Board::empty_board();
    let (blue_pieces, red_pieces) = e.serialize();
    
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
        Ok(match_id as u64)
    } else {
        Err(Error::NotCompleted(NotCompletedReason::UnrecoverableError))
    }
}

#[cfg(test)]
mod test;
    

