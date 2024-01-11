use clap::{Parser, Subcommand};
use std::{fs, io};

#[derive(Debug, Parser)]
struct Cli {
    ///The command to run
    #[clap(subcommand)]
    cmd: Command,       

}

#[derive(Debug, Subcommand)]
enum Command {
    ///list files and directories
    Ls {
        ///Show hidden files also
        #[clap(short='a')]
        a: bool,
    },
    ///Example command for testing
    Ex {
        b: String,
    }
}

fn main() {
    
    let cli = Cli::parse();
    match &cli.cmd {
        Command::Ls{a} => {
            println!("Listing {:?}", a);
            match ls(*a) {
                Ok(()) => {},
                Err(err) => {
                    println!("Error {}", err);
                },
            }
        }
        Command::Ex{b} => println!("Example command {}", b),
    }
    //println!("{:?}", cli);
}

//TODO support specific dir later
fn ls(hidden: bool) -> io::Result<()> {
    for entry in fs::read_dir(".")? {
        let dir = entry?;
        let name = dir.file_name().into_string();
        match name {
            Ok(p) => {
                if hidden || !p.starts_with(".") {
                    println!("{}", p);
                }
            },
            Err(_) => todo!(),
        }
    }
    Ok(())
}
