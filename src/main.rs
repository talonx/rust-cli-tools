use clap::{Parser, Subcommand};
use regex::Regex;
use std::{
    fs::{self, File},
    io::{BufRead, BufReader, Error as IOError, ErrorKind, Read, Result as IOResult, Write},
    path::PathBuf,
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
    Ps {},
    ///example command for testing
    Ex {
        b: String,
    },
}

fn main() {
    let cli = Cli::parse();
    let res = match &cli.cmd {
        Command::Ls { a, file } => ls(*a, file),
        Command::Cp { source, target } => copy(source, target),
        Command::Ps {} => ps(),
        Command::Ex { .. } => Ok(()),
    };
    match res {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error: {}", e);
            //TODO https://rust-cli.github.io/book/in-depth/exit-code.html
            std::process::exit(1);
        }
    };
    //println!("{:?}", cli);
}

//https://docs.kernel.org/filesystems/proc.html
fn ps() -> IOResult<()> {
    let self_stat = File::open("/proc/self/status")?;
    let bufreader = BufReader::new(self_stat);
    let mut my_uid = String::from("");
    for line in bufreader.lines() {
        match line.unwrap() {
            line if line.starts_with("Uid:") => {
                my_uid = line.split('\t').nth(1).unwrap().to_owned();
                break;
            }
            _ => {}
        }
    }
    if my_uid == "" {
        panic!("No self uid found");
    }
//    println!("Found uid {:?}", my_uid);
    println!("PID    TTY    TIME    CMD");
//    let my_uid_str = my_uid.as_str;
    let re = Regex::new(r"[0-9]+").unwrap();
    for entry in fs::read_dir("/proc")? {
        let dir = entry?;
        if dir.metadata()?.is_dir() {
            let name = dir.file_name().into_string().unwrap();
            if re.is_match(name.as_str()) {
                let (uid, cmd) = get_proc_status(&name);
                if my_uid == uid {
                    println!("{} {} {} {}", name, get_tty(&name), get_time(&name), cmd);
                }
            }
        }
    }
    Ok(())
}

fn get_proc_status(pid_dir: &str) -> (String, String) {
    let mut uid = String::from("");
    let mut cmd = String::from("");
    match File::open("/proc/".to_owned() + pid_dir + "/status") {
        Ok(status_f) => {
            let bufreader = BufReader::new(status_f);
            for line in bufreader.lines() {
                match line.unwrap() {
                    line if line.starts_with("Uid:") => {
                        uid = line.split('\t').nth(1).unwrap().to_owned();
                    }
                    line if line.starts_with("Name:") => {
                        cmd = line.split('\t').nth(1).unwrap().to_owned();
                    }
                    std::string::String {..} => {},
                }
            }
        }
        Err(err) => panic!("Unable to read proc filesystem: {}", err),
    }
    (uid, cmd)
}

fn get_time(file: &str) -> &str {
    "00:00:00"
}

fn get_tty(pid_dir: &str) -> String {
    match File::open("/proc/".to_owned() + pid_dir + "/stat") {
        Ok(stat_f) => {
            let bufreader = BufReader::new(stat_f);
            match bufreader.lines().next() {
                None => panic!("No lines in stat file for {}", pid_dir),
                Some(line) => line.expect("Failed to read stat file").split(' ').nth(6).unwrap().to_owned()
            }
        },
        Err(err) => panic!("Unable to read the proc filesystem: {}", err),
    }
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
                    Err(error) => match error.kind() {
                        ErrorKind::Interrupted => {}
                        _ => return Err(error),
                    },
                    Ok(0) => return Ok(()), //Not sure if this is right
                    Ok(m) => {
                        if m < n {
                            return Err(IOError::new(
                                ErrorKind::Other,
                                format!("wrote {} but had {}", m, n),
                            ));
                        }
                    }
                }
            }
            Err(error) => match error.kind() {
                ErrorKind::Interrupted => {}
                _ => return Err(error),
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
