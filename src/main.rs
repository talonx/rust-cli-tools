use clap::{Parser, Subcommand};
use std:: {
    path::PathBuf,
    fs::{self, File},
    io::{Read, Result as IOResult, ErrorKind, Write, Error as IOError},
};

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
        #[clap(short = 'a')]
        a: bool,
        ///File or directory
        file: Option<String>,
    },
    ///copy a file
    Cp {
        ///the file to copy from
        source: String,
        ///the target file to copy to
        target: String,
    },
    ///example command for testing
    Ex { b: String },
}

fn main() {
    let cli = Cli::parse();
    let res = match &cli.cmd {
        Command::Ls { a, file } => ls(*a, file),
        Command::Cp {source, target} => copy(source, target),
        Command::Ex {..} => Ok(()),
    };
    match res {
        Ok(_) => {},
        Err(e) => {
            eprintln!("Error: {}", e);
            //TODO https://rust-cli.github.io/book/in-depth/exit-code.html
            std::process::exit(1);
        },
    };
    //println!("{:?}", cli);
}

fn copy(source: &String, target: &String) -> IOResult<()> {
    let mut buf = [0; 8192];
    let mut f_source = File::open(source)?;
    let mut f_target = File::create(target)?;

    loop {
        match f_source.read(&mut buf) {
            Ok(0) => return Ok(()),
            Ok(n) => {
                match f_target.write(&buf[0..n]) {
                    Err(error) => {
                        match error.kind() {
                            ErrorKind::Interrupted => {},
                            _ => return Err(error),
                        }
                    },
                    Ok(0) => return Ok(()),//Not sure if this is right
                    Ok(m) => {
                        if m < n {
                            return Err(IOError::new(ErrorKind::Other, format!("wrote {} but had {}", m, n)));
                        }
                    }
                }
            },
            Err(error) => {
                match error.kind() {
                    ErrorKind::Interrupted => {},
                    _ => return Err(error),
                }
            },
        }
    }
    //TODO explicit flush to catch errors
}

fn ls(hidden: bool, file: &Option<String>) -> IOResult<()> {
    let mut target = ".";
    match file {
        Some(file_or_dir) => target = file_or_dir,
        None => {}
    }

    if PathBuf::from(target).is_dir() {
        for entry in fs::read_dir(target)? {
            let dir = entry?;
            let name = dir.file_name().into_string();
            match name {
                Ok(p) => {
                    if hidden || !p.starts_with(".") {
                        println!("{}", p);
                    }
                }
                Err(_) => todo!(),
            }
        }
    } else {
        //Until we have metadata support
        println!("{}", target);
    }
    Ok(())
}
