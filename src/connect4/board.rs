use std::cmp;
use std::string::String;

const COLUMNS : [u64; 7] = 
    [ 0x1u64
    , 0x40u64
    , 0x1000u64
    , 0x40000u64
    , 0x1000000u64
    , 0x40000000u64
    , 0x1000000000u64 ];

const TURN_INDICATOR : u64 = 0x40000000000u64;

const BOARD_HEIGHT : u8 = 6;
const BOARD_WIDTH : u8 = 7;


#[derive(Debug, PartialEq, Eq)]
pub enum Slot {
    EMPTY,
    RED,
    BLUE
}

/* 
  The board is represented as 2 u64, one indicating the position of the red pieces
  and one indicating the positions of the blue pieces. A piece being present is indicated with a
  1 in the corresponding position (and its absence with a 0). 
  
  The positions are assigned, LSB first, from the bottom left of the board, and going upwards. 
  
  The bit at position 42 indicates the turn. Both ints have a 0 there when it's red's turn, and
  both have a 1 there when it is blue's turn.
  
  The bit at position 43 indicates game over. The bit is set on the pieces of the winning player,
  or in both in the case of a tie.
*/

#[derive(Debug, PartialEq, Eq)]
pub struct Board {
    red_pieces : u64,
    blue_pieces : u64
}

impl Board {
    fn drop_piece(&mut self, column : u8, piece : Slot) -> Result<(), ()> {
        assert!(column < BOARD_WIDTH);
        assert!(piece != Slot::EMPTY);
    
        let mut row = COLUMNS[column as usize];
        let limit = row << BOARD_HEIGHT;
        let occupied = self.red_pieces | self.blue_pieces;
        while (row & occupied) != 0 && row != limit {
            row = row << 1;
        }
        
        if row < limit {
            match piece {
                Slot::RED => self.red_pieces = self.red_pieces | row,
                Slot::BLUE => self.blue_pieces = self.blue_pieces | row,
                Slot::EMPTY => panic!("active player is empty")
            };
            Ok(())
        } else {
            Err(())
        }
    }
    
    pub fn play_move(&mut self, column : u8) {
        match self.drop_piece(column, self.active_player()) {
            Ok(_) => self.swap_turn(),
            Err(_) => {}
        }
    }
    
    pub fn empty_board() -> Board {
        Board {red_pieces : 0, blue_pieces : 0}
    }
    
    pub fn slot_at(&self, x : u8, y : u8) -> Slot {
        assert!(x < BOARD_WIDTH && y < BOARD_HEIGHT);
        
        let location = COLUMNS[x as usize] << y;
        if (self.red_pieces & location) != 0 {
            Slot::RED
        } else if (self.blue_pieces & location) != 0 {
            Slot::BLUE
        } else {
            Slot::EMPTY
        }
    }
    
    pub fn active_player(&self) -> Slot {
        if self.red_pieces & TURN_INDICATOR != 0 {
            Slot::BLUE
        }
        else {
            Slot::RED
        }
    }
    
    fn swap_turn(&mut self) {
        self.red_pieces = self.red_pieces ^ TURN_INDICATOR;
        self.blue_pieces = self.blue_pieces ^ TURN_INDICATOR;
    }
    
    pub fn display(&self, red : &str, blue : &str, empty : &str, 
        separator : &str, line_start : &str, line_end : &str, line_separator : &str) -> String {
        let (bh, bw) = (BOARD_HEIGHT as usize, BOARD_WIDTH as usize);
        let output_length
            = bh*bw*cmp::max(red.len(), cmp::max(blue.len(), empty.len()))
            + bh*line_start.len()
            + bh*line_end.len()
            + bh*(bw - 1)*separator.len()
            + (bh + 1)*line_separator.len();
        let mut output_string = String::with_capacity(output_length);

        for y in 0..BOARD_HEIGHT {
            output_string.push_str(line_separator);
            output_string.push_str(line_start);
            for x in 0..(BOARD_WIDTH-1) {
                output_string.push_str(
                    match self.slot_at(x, BOARD_HEIGHT - y - 1) {
                        Slot::RED => red,
                        Slot::BLUE => blue,
                        Slot::EMPTY => empty
                    }
                );
                output_string.push_str(separator);
            }
            output_string.push_str(
                match self.slot_at(BOARD_WIDTH - 1, BOARD_HEIGHT - y - 1) {
                    Slot::RED => red,
                    Slot::BLUE => blue,
                    Slot::EMPTY => empty
                }
            );
            output_string.push_str(line_end);
        }
        output_string.push_str(line_separator);
        
        output_string
    }
}


#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn empty_is_empty() {
        let e = Board::empty_board();
        for i in 0..7 {
            for j in 0..6 {
                assert_eq!(e.slot_at(i, j), Slot::EMPTY);
            }
        }
    }
    
    #[test]
    #[should_panic]
    fn out_of_bounds_index_h() {
        let e = Board::empty_board();
        let _x = e.slot_at(7,0);
    }
    
    #[test]
    #[should_panic]
    fn out_of_bounds_index_v() {
        let e = Board::empty_board();
        let _x = e.slot_at(0,6);
    }
    
    #[test]
    fn example_game() {
        let mut e = Board::empty_board();
        assert_eq!(e.active_player(), Slot::RED);
        assert_eq!(e.slot_at(2,0), Slot::EMPTY);
        assert_eq!(e.slot_at(3,0), Slot::EMPTY);
        assert_eq!(e.slot_at(2,1), Slot::EMPTY);
        e.play_move(2);
        assert_eq!(e.active_player(), Slot::BLUE);
        assert_eq!(e.slot_at(2,0), Slot::RED);
        assert_eq!(e.slot_at(3,0), Slot::EMPTY);
        assert_eq!(e.slot_at(2,1), Slot::EMPTY);
        e.play_move(3);
        assert_eq!(e.active_player(), Slot::RED);
        assert_eq!(e.slot_at(2,0), Slot::RED);
        assert_eq!(e.slot_at(3,0), Slot::BLUE);
        assert_eq!(e.slot_at(2,1), Slot::EMPTY);
        e.play_move(2);
        assert_eq!(e.active_player(), Slot::BLUE);
        assert_eq!(e.slot_at(2,0), Slot::RED);
        assert_eq!(e.slot_at(3,0), Slot::BLUE);
        assert_eq!(e.slot_at(2,1), Slot::RED);
        e.play_move(4);
        assert_eq!(e.active_player(), Slot::RED);
        assert_eq!(e.slot_at(2,0), Slot::RED);
        assert_eq!(e.slot_at(3,0), Slot::BLUE);
        assert_eq!(e.slot_at(2,1), Slot::RED);
    }
    
}

