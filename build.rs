extern crate cc;

fn main() {
    println!("cargo:rerun-if-changed=src/connect4/ai_c_files/board.c");
    println!("cargo:rerun-if-changed=src/connect4/ai_c_files/board.h");
    println!("cargo:rerun-if-changed=src/connect4/ai_c_files/montecarlo.c");
    println!("cargo:rerun-if-changed=src/connect4/ai_c_files/montecarlo.h");
    println!("cargo:rerun-if-changed=src/connect4/ai_c_files/stack.c");
    println!("cargo:rerun-if-changed=src/connect4/ai_c_files/stack.h");
    println!("cargo:rerun-if-changed=src/connect4/ai_c_files/called_from_rust.c");

    cc::Build::new()
        .file("src/connect4/ai_c_files/stack.c")
        .file("src/connect4/ai_c_files/board.c")
        .file("src/connect4/ai_c_files/montecarlo.c")
        .file("src/connect4/ai_c_files/called_from_rust.c")
        .compile("cfiles");
}
