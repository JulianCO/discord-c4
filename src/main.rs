mod connect4;

use connect4::board::Board;
use connect4::monte_carlo_ai;

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
    
    let suggested_move = monte_carlo_ai::ai_move(&b, 32768);
    
    match suggested_move {
        Ok(x) => 
            println!("The ai suggest playing the {} column", x+1),
        Err(_) =>
            println!("The AI failed!")
    };
}


