#[derive(Debug, PartialEq, Eq)]
enum Slot {
    EMPTY,
    RED,
    BLUE
}

#[derive(Debug, PartialEq, Eq)]
struct Board {
    yellow_pieces : u64,
    blue_pieces : u64
}

impl Board {
    fn play_move(&mut self, column : u8) {
    }
    
    fn empty_board() -> Board {
        Board {yellow_pieces : 0, blue_pieces : 0}
    }
    
    fn slot_at(&self, x : u8, y : u8) -> Slot {
        Slot::EMPTY
    }
}



fn main() {
    println!("Hello, world!");
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
        let x = e.slot_at(7,0);
    }
    
    #[test]
    #[should_panic]
    fn out_of_bounds_index_v() {
        let e = Board::empty_board();
        let x = e.slot_at(0,6);
    }
    
    #[test]
    fn example_game() {
        let mut e = Board::empty_board();
        assert_eq!(e.slot_at(2,0), Slot::EMPTY);
        assert_eq!(e.slot_at(3,0), Slot::EMPTY);
        assert_eq!(e.slot_at(2,1), Slot::EMPTY);
        e.play_move(2);
        assert_eq!(e.slot_at(2,0), Slot::RED);
        assert_eq!(e.slot_at(3,0), Slot::EMPTY);
        assert_eq!(e.slot_at(2,1), Slot::EMPTY);
        e.play_move(3);
        assert_eq!(e.slot_at(2,0), Slot::RED);
        assert_eq!(e.slot_at(3,0), Slot::BLUE);
        assert_eq!(e.slot_at(2,1), Slot::EMPTY);
        e.play_move(2);
        assert_eq!(e.slot_at(2,0), Slot::RED);
        assert_eq!(e.slot_at(3,0), Slot::BLUE);
        assert_eq!(e.slot_at(2,1), Slot::RED);
        e.play_move(4);
        assert_eq!(e.slot_at(2,0), Slot::RED);
        assert_eq!(e.slot_at(3,0), Slot::EMPTY);
        assert_eq!(e.slot_at(2,1), Slot::EMPTY);
    }
    
}

