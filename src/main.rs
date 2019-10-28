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
    pub static ref BUILDIN_FUNCTIONS: HashMap<&'static str, fn(Vec<&str>) -> i32> = [
        ("cd",   buildin_cd   as fn(Vec<&str>) -> i32),
        ("help", buildin_help as fn(Vec<&str>) -> i32),
        ("exit", buildin_exit as fn(Vec<&str>) -> i32)
    ].iter().cloned().collect();
}

fn buildin_cd(args: Vec<&str>) -> i32 {
    if args.len() < 2 {
        println!("lsh: expected argument to \"cd\"");
        return 1;
    }
    match chdir(args[1]) {
        Ok(_) => println!("moved");
        Err(_) => println!("move failed");
    }
    return 1
}

fn buildin_help(_args: Vec<&str>) -> i32 {
    println!("esh v0.01");
    BUILDIN_FUNCTIONS.iter().for_each(|(name, _)| {
        println!("- {}", name)
    });
    return 1
}

fn buildin_exit(_args: Vec<&str>) -> i32 {
    return 0
}

fn esh_launch(args: Vec<&str>) -> i32 {
    fn wait_loop(pid: Pid) { // pidのプロセスが終了するまでループ
        match waitpid(pid, Some(WaitPidFlag::WUNTRACED)) {
            Ok(WaitStatus::Exited(_, _)) => println!("exited"),
            Ok(WaitStatus::Signaled(_, _, _)) => println!("signaled"),
            _ => wait_loop(pid),
        }
    }

    match fork() {
        Ok(ForkResult::Parent {child, .. }) => {
            wait_loop(child)
        }
        Ok(ForkResult::Child) => {
            let argsc: Vec<CString> = args.iter().map(|&arg|{ CString::new(arg).unwrap() }).collect();
            execvp(&argsc[0], &argsc);
            exit(1);
        }
        Err(_) => println!("fork failed"),
    }
    return 1
}

fn esh_execute(args: Vec<&str>) -> i32 {
    if args.len() == 0 { return 1 };
    match BUILDIN_FUNCTIONS.get(args[0]) {
     Some(f) => f(args),
     None => esh_launch(args),
    }
}

fn esh_loop() {
    print!("$ ");
    io::stdout().flush().unwrap();
    let mut line = String::new();
    let _ = io::stdin().read_line(&mut line);
    let args: Vec<&str> = line.split_whitespace().collect();
    let status: i32     = esh_execute(args);
    if status != 0 { esh_loop(); }
}

fn main() {
    println!("esh v0.01");
    esh_loop();            
}
