use std::env;
use std::io;
use std::result::Result;
use std::io::prelude::*;

const DEBUG: bool = true;

fn do_login(args: Vec<String>) -> Result<(), &'static str> {
    println!("Loggin in...");
    Ok(())
}

fn parse_args(args: Vec<String>) -> Result<(), &'static str> {
    if args.len() < 2 {
        return Err("Need a command!");
    }

    match args[1].as_ref() {
        "login"  => do_login(args),
         _ => Err("Unknown command {} not implemented"),
    }
}

fn main() -> Result<(), &'static str> {
    let args: Vec<String> = env::args().collect();
    println!("\n\n\n\n");
    println!("Rusty Cache starting up!...");

    for arg in &args {
        println!("\t{}", arg);
    }
    parse_args(args)?;

    println!("Rusty Cache exiting Exiting");
    Ok(())
}
