extern crate cc;

fn main() { 
    println!("cargo:rerun-if-changed=src/connect4/cfile1.c");
    println!("cargo:rerun-if-changed=src/connect4/cfile2.c");

    cc::Build::new()
        .file("src/connect4/cfile1.c")
        .file("src/connect4/cfile2.c")
        .compile("cfiles");
}
