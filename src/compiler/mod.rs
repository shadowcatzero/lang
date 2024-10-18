use std::{
    fs::{create_dir_all, OpenOptions},
    os::unix::fs::OpenOptionsExt,
    path::Path,
    process::Command,
};

mod elf;
mod program;
mod riscv64;
mod target;

pub use program::*;

pub fn main() {
    use std::io::prelude::*;
    let dir = Path::new("./build");
    create_dir_all(dir).expect("Failed to create or confirm build directory");
    let name = Path::new("test");
    let path = dir.join(name);
    let path = path.as_os_str();
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .mode(0o750)
        .open(path)
        .expect("Failed to create file");
    file.write_all(&riscv64::gen())
        .expect("Failed to write to file");
    file.sync_all().expect("Failed to sync file");
    let mut p = Command::new("qemu-riscv64");
    let run_gdb = std::env::args().nth(1).is_some_and(|a| a == "d");
    let proc = if run_gdb {
        p.arg("-g").arg("1234").arg(path).spawn()
    } else {
        p.arg(path).spawn()
    };
    if let Ok(mut process) = proc {
        let mut print_exit = true;
        if run_gdb {
            match Command::new("gdb")
                .arg("-q")
                .arg("-ex")
                .arg("target remote :1234")
                .arg(path)
                .spawn()
            {
                Ok(mut gdb) => {
                    gdb.wait().expect("xd");
                }
                Err(e) => {
                    print_exit = false;
                    println!("gdb error: {e:?}");
                    process.kill().expect("uh oh");
                }
            }
        }
        if let Ok(status) = process.wait() {
            if print_exit && status.code().is_none_or(|c| c != 0) {
                println!("{}", status);
            }
        }
    }
}

