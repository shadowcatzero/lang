use std::{
    fs::{create_dir_all, OpenOptions},
    os::unix::fs::OpenOptionsExt,
    path::Path,
    process::Command,
};

mod riscv64;
mod program;
mod target;

pub fn main() {
    use std::io::prelude::*;
    let dir = Path::new("build");
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
    if let Ok(mut process) = Command::new("qemu-riscv64").arg(path).spawn() {
        if let Ok(status) = process.wait() {
            if status.code().is_none_or(|c| c != 0) {
                println!("{}", status);
            }
        }
    }
}

//     qemu-riscv64 -g 1234 test &
//     riscv64-linux-gnu-gdb -q \
//         -ex "target remote :1234" \
//         test

