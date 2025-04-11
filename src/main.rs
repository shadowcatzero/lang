#![feature(box_patterns)]
#![feature(try_trait_v2)]
#![feature(trait_alias)]
#![feature(let_chains)]
// dawg what
#![feature(str_as_str)]

use ir::{LProgram, UProgram};
use parser::{NodeParsable, PModule, PStatement, ParserCtx};
use util::Labelable;
use std::{
    fs::{create_dir_all, OpenOptions},
    io::{stdout, BufRead, BufReader},
    os::unix::fs::OpenOptionsExt,
    path::Path,
    process::Command,
};

mod common;
mod compiler;
mod ir;
mod parser;
mod util;

fn main() {
    let file = std::env::args_os().nth(1);
    // TODO: professional arg parsing
    let gdb = std::env::args().nth(2).is_some_and(|a| a == "--debug");
    let asm = std::env::args().nth(2).is_some_and(|a| a == "--asm");
    if let Some(path) = file {
        let file = std::fs::read_to_string(path).expect("failed to read file");
        run_file(&file, gdb, asm);
    } else {
        run_stdin();
    }
}

fn run_file(file: &str, gdb: bool, asm: bool) {
    let mut ctx = ParserCtx::from(file);
    let res = PModule::parse_node(&mut ctx);
    let mut output = ctx.output;
    'outer: {
        if !output.errs.is_empty() {
            break 'outer;
        }
        // println!("Parsed:");
        // println!("{:#?}", res.node);
        let Some(module) = res.node.as_ref() else {
            break 'outer;
        };
        let mut program = UProgram::new();
        module.lower(&mut program, &mut output);
        if !output.errs.is_empty() {
            break 'outer;
        }
        program.resolve_types();
        // println!("vars:");
        // for (id, def) in program.iter_vars() {
        //     println!("    {id:?} = {}: {}", program.names.get(id), program.type_name(&def.ty));
        // }
        // for (id, f) in program.iter_fns() {
        //     println!("{}:{id:?} = {:#?}", program.names.get(id), f);
        // }
        output = program.validate();
        if !output.errs.is_empty() {
            break 'outer;
        }
        output.write_for(&mut stdout(), file);
        let program = LProgram::create(&program).expect("morir");
        let unlinked = compiler::compile(&program);
        if asm {
            println!("{:?}", unlinked);
        } else {
            let bin = unlinked.link().to_elf();
            println!("compiled");
            save_run(&bin, gdb);
        }
    }
    output.write_for(&mut stdout(), file);
}

fn save_run(binary: &[u8], run_gdb: bool) {
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
    file.write_all(binary).expect("Failed to write to file");
    file.sync_all().expect("Failed to sync file");
    println!("running...");
    let mut p = Command::new("qemu-riscv64");
    let proc = if run_gdb {
        p.arg("-g").arg("1234").arg(path).spawn()
    } else {
        p.arg(path).spawn()
    };
    if let Ok(mut process) = proc {
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
                    println!("gdb error: {e:?}");
                    process.kill().expect("uh oh");
                }
            }
        }
        if let Ok(status) = process.wait() {
            if let Some(code) = status.code() {
                std::process::exit(code);
            }
        }
    }
}

pub fn run_stdin() {
    for line in BufReader::new(std::io::stdin()).lines() {
        let str = &line.expect("failed to read line");
        let mut ctx = ParserCtx::from(&str[..]);
        if let Some(expr) = PStatement::parse_node(&mut ctx).node.as_ref() {
            if ctx.next().is_none() {
                println!("{:?}", expr);
            } else {
                println!("uhhhh ehehe");
            }
        }
        ctx.output.write_for(&mut stdout(), str);
    }
}
