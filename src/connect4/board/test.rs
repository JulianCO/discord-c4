use super::*;

fn test_game(moves : &[u8], expected_result : GameResult) {
    let mut e = Board::empty_board();
    for k in moves {
        assert!(e.is_move_legal(*k));
        e.play_move(*k);
    }
    assert!(e.game_status() == GameStatus::GameOver(expected_result));
}

#[test]
fn empty_is_empty() {
    let e = Board::empty_board();
    for i in 0..7 {
        for j in 0..6 {
            assert_eq!(e.slot_at(i, j), Slot::Empty);
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
fn example_game_1() {
    let mut e = Board::empty_board();
    assert_eq!(e.game_status(), GameStatus::Turn(Player::Red));
    assert_eq!(e.slot_at(2,0), Slot::Empty);
    assert_eq!(e.slot_at(3,0), Slot::Empty);
    assert_eq!(e.slot_at(2,1), Slot::Empty);
    e.play_move(2);
    assert_eq!(e.game_status(), GameStatus::Turn(Player::Blue));
    assert_eq!(e.slot_at(2,0), Slot::Piece(Player::Red));
    assert_eq!(e.slot_at(3,0), Slot::Empty);
    assert_eq!(e.slot_at(2,1), Slot::Empty);
    e.play_move(3);
    assert_eq!(e.game_status(), GameStatus::Turn(Player::Red));
    assert_eq!(e.slot_at(2,0), Slot::Piece(Player::Red));
    assert_eq!(e.slot_at(3,0), Slot::Piece(Player::Blue));
    assert_eq!(e.slot_at(2,1), Slot::Empty);
    e.play_move(2);
    assert_eq!(e.game_status(), GameStatus::Turn(Player::Blue));
    assert_eq!(e.slot_at(2,0), Slot::Piece(Player::Red));
    assert_eq!(e.slot_at(3,0), Slot::Piece(Player::Blue));
    assert_eq!(e.slot_at(2,1), Slot::Piece(Player::Red));
    e.play_move(4);
    assert_eq!(e.game_status(), GameStatus::Turn(Player::Red));
    assert_eq!(e.slot_at(2,0), Slot::Piece(Player::Red));
    assert_eq!(e.slot_at(3,0), Slot::Piece(Player::Blue));
    assert_eq!(e.slot_at(2,1), Slot::Piece(Player::Red));
}

#[test]
fn filling_up_a_column() {
    let mut e = Board::empty_board();
    assert!(e.is_move_legal(2));
    e.play_move(2);
    assert!(e.is_move_legal(2));
    e.play_move(2);
    assert!(e.is_move_legal(2));
    e.play_move(2);
    assert!(e.is_move_legal(2));
    e.play_move(2);
    assert!(e.is_move_legal(2));
    e.play_move(2);
    assert!(e.is_move_legal(2));
    e.play_move(2);
    assert!(!e.is_move_legal(2));
    assert!(e.is_move_legal(3));
}

#[test]
fn test_game_1() {
    let game = vec![3, 3, 2, 1, 3, 5, 2, 2, 4, 3, 1, 2, 5, 1, 1, 3, 5, 5, 5, 2, 1, 2];
    test_game(&game, GameResult::Winner(Player::Blue));
}

