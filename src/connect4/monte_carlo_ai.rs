use super::board::Board;
use super::board::GameStatus;

extern {
    fn __c_montecarlo_c4_ai(red_pieces : u64, blue_pieces : u64, tree_size: u32) -> u8;
}

pub fn ai_move(b : &Board, rollout_number : u32) -> Result<u8, ()> {
    match b.game_status() {
        GameStatus::GameOver(_) => Err(()),
        GameStatus::Turn(_) => {
            let (red_pieces, blue_pieces) = b.serialize();
            let imported_ai_move : u8 = unsafe {
                __c_montecarlo_c4_ai(red_pieces, blue_pieces, rollout_number)
            };
            if imported_ai_move == 7 {
                Err(())
            }
            else {
                Ok(imported_ai_move)
            }
        }
    }
}


