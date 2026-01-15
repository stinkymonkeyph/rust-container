use anyhow::Result;
use nix::mount::{mount, MsFlags};
use nix::sched::{unshare, CloneFlags};
use nix::unistd::{fork, sethostname, ForkResult};
use std::process::Command;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} run <command> [args...]", args[0]);
        std::process::exit(1);
    }

    if args[1] == "run" {
        if args.len() < 3 {
            eprintln!("Usage: {} run <command> [args...]", args[0]);
            std::process::exit(1);
        }
        run(&args[2..])?;
    } else {
        eprintln!("Unknown command: {}", args[1]);
        std::process::exit(1);
    }

    Ok(())
}

fn run(command_args: &[String]) -> Result<()> {
    println!("Running {:?} as PID {}", command_args, std::process::id());

    let flags = CloneFlags::CLONE_NEWUTS   // hostname isolation
        | CloneFlags::CLONE_NEWPID         // PID isolation
        | CloneFlags::CLONE_NEWNS;         // mount isolation

    unshare(flags)?;

    // Fork a child process
    match unsafe { fork()? } {
        ForkResult::Parent { child } => {
            // Parent process - wait for child
            println!("Container started with PID: {}", child);
            nix::sys::wait::waitpid(child, None)?;
            println!("Container exited");
        }
        ForkResult::Child => {
            // Child process - this runs in the new namespaces
            child(command_args)?;
        }
    }

    Ok(())
}

fn child(command_args: &[String]) -> Result<()> {
    println!("Inside container as PID {}", std::process::id());

    sethostname("container")?;
    setup_proc()?;

    let status = Command::new(&command_args[0])
        .args(&command_args[1..])
        .status()?;

    std::process::exit(status.code().unwrap_or(1));
}

fn setup_proc() -> Result<()> {
    // Mount new proc filesystem
    // mount(source, target, fstype, flags, data)
    mount(
        Some("proc"),           // source - special name for proc
        "/proc",                // target - where to mount
        Some("proc"),           // filesystem type
        MsFlags::empty(),       // mount flags
        None::<&str>,          // additional data
    )?;

    println!("Mounted new /proc filesystem");
    
    Ok(())
}
