mod connect4;

use crate::connect4::board::Board;

extern {
    fn times_pi_rounded(n: u32) -> u32;
    fn initialize_module();
    fn flip_coin() -> u8;
}

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
    
    unsafe {
        initialize_module();
    }
    let seventy_two;
    unsafe {
        seventy_two = times_pi_rounded(23);
    }
    println!("{}", seventy_two);
    let mut x;
    for _i in 0..10 {
        unsafe {
            x = flip_coin();
        }
        if x == 0 {
            println!("Heads!");
        } else {
            println!("Tails!");
        }
    }
    
}


