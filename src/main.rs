mod connect4;

use crate::connect4::board::Board;


fn main() {
    let mut b = Board::empty_board();
    b.play_move(3);
    b.play_move(2);
    b.play_move(3);
    b.play_move(3);
    b.play_move(3);
    b.play_move(4);
    b.play_move(4);
    
    let example_board = b.display("r", "b", " ", "|", "|", "|\n", "+-+-+-+-+-+-+-+\n");
    let example_discord = b.display(":red_circle:", ":blue_circle:", ":white_circle:", "", "", "\n", "");

    print!("{}", example_board);
    println!("");
    print!("{}", example_discord);
}


