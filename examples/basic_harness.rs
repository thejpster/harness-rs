extern crate harness;

use std::io::Read;

fn foo() -> Result<(), &'static str> {
    println!("Called foo!");
    Ok(())
}

fn bar() -> Result<(), &'static str> {
    println!("Called bar!");
    Err("bar doesn't work")
}

fn quit() -> Result<(), &'static str> {
    std::process::exit(0)
}

fn main() {
    println!("Command line harness example\r\n");
    let mut h = harness::Harness::new(std::io::stdout());
    h.add_command("foo", "Foo's the frobble", foo);
    h.add_command("bar", "Bar's the frobble", bar);
    h.add_command("quit", "Exit's the program", quit);
    h.prompt().unwrap();
    loop {
        let mut buf = [0u8; 1];
        if let Ok(n) = std::io::stdin().read(&mut buf) {
            for b in buf.iter().take(n) {
                h.receive_and_print(*b).unwrap();
            }
        }
    }
}