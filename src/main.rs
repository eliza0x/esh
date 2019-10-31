#[macro_use]
extern crate lazy_static;
extern crate nix;

use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::unistd::{fork, ForkResult, Pid, chdir, execvp};
use std::collections::HashMap;
use std::ffi::CString;
use std::io::{self, Write};
use std::process::exit;

lazy_static! {
    pub static ref BUILDIN_FUNCTIONS: HashMap<&'static str, fn(&Vec<&str>)> = [
        ("cd",   buildin_cd   as fn(&Vec<&str>)),
        ("help", buildin_help as fn(&Vec<&str>)),
        ("exit", buildin_exit as fn(&Vec<&str>))
    ].iter().cloned().collect();
}

fn buildin_cd(args: &Vec<&str>) {
    if args.len() < 2 {
        println!("lsh: expected argument to \"cd\"");
        return;
    }
    match chdir(args[1]) {
        Ok(_) => println!("moved"),
        Err(_) => println!("move failed")
    }
}

fn buildin_help(_args: &Vec<&str>) {
    println!("esh v0.01");
    BUILDIN_FUNCTIONS.iter().for_each(|(name, _)| {
        println!("- {}", name)
    });
}

fn buildin_exit(_args: &Vec<&str>) {
    exit(0);
}

fn wait_loop(pid: Pid) -> Result<i32, String> { // pidのプロセスが終了するまでループ
    match waitpid(pid, Some(WaitPidFlag::WUNTRACED)) {
        Ok(WaitStatus::Exited(_, status)) => Ok(status),
        Ok(WaitStatus::Signaled(_, _, _)) => Err("signaled".to_string()),
        _ => wait_loop(pid),
    }
}

fn esh_launch(args: &Vec<&str>) {
    let argsc: Vec<CString> = args.iter().map(|&arg|{ CString::new(arg).unwrap() }).collect();
    if let Err(_) = execvp(&argsc[0], &argsc) {
        println!("execvp failed.");
    }
}

fn esh_execute(commands: Vec<Vec<&str>>) -> Result<i32, String> {
    let commands_index: usize = 0;
    let command: &Vec<&str> = &commands[commands_index];
    match fork() {
        Ok(ForkResult::Parent {child, .. }) => {
            wait_loop(child)
        }
        Ok(ForkResult::Child) => {
            match BUILDIN_FUNCTIONS.get(command[0]) {
             Some(f) => f(command),
             None => esh_launch(command),
            }
            exit(1);
        }
        Err(_) => Err("fork failed".to_string()),
    }
}

fn esh_loop() {
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();
        let mut line = String::new();
        let _ = io::stdin().read_line(&mut line);
        let commands: Vec<Vec<&str>> = line.split("|").map(|c| c.split_whitespace().collect()).collect();
        let status: Result<i32, String> = esh_execute(commands);
        match status {
            Ok(status) => {
                println!("{}", status);
            }
            Err(e) => {
                println!("{}", e);
            }
        }
    }
}

fn main() {
    println!("esh v0.01");
    esh_loop();            
}
