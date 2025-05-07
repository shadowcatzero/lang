#![feature(box_patterns)]
#![feature(try_trait_v2)]
#![feature(trait_alias)]
#![feature(let_chains)]
#![feature(iterator_try_collect)]
// dawg what
#![feature(str_as_str)]

pub const FILE_EXT: &str = "lang";

use common::{CompilerOutput, SrcFile};
use ir::{LProgram, UProgram};
use parser::{Import, Imports, PModule, ParserCtx};
use std::{
    collections::HashSet,
    fs::{create_dir_all, OpenOptions},
    io::stdout,
    os::unix::fs::OpenOptionsExt,
    path::{Path, PathBuf},
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
        let path = PathBuf::from(path);
        run_file(&path, gdb, asm);
    } else {
        run_stdin();
    }
}

impl UProgram {
    pub fn from_path(path: &Path) -> (Self, CompilerOutput) {
        let parent = path.parent().expect("bruh");
        let mut program = Self::new();
        let mut output = CompilerOutput::new();

        let mut imports = Imports::new();
        imports.insert(Import(vec![path
            .file_name()
            .expect("bruh")
            .to_str()
            .expect("bruh")
            .to_string()]));
        let mut imported = HashSet::new();
        let mut fid = 0;

        while !imports.is_empty() {
            let iter = std::mem::take(&mut imports);
            for i in iter {
                let import_path = &i.0;
                if imported.contains(&i) {
                    continue;
                }
                let mut file_path = parent.to_path_buf();
                file_path.extend(import_path);
                file_path.set_extension(FILE_EXT);
                let text = std::fs::read_to_string(&file_path).expect("failed to read file");
                output.file_map.insert(
                    fid,
                    SrcFile {
                        path: file_path,
                        text: text.clone(),
                    },
                );
                let mut ctx = ParserCtx::new(fid, text.as_str(), &mut output);
                fid += 1;
                let res = PModule::parse(&mut ctx);
                // println!("Parsed:");
                // println!("{:#?}", res.node);
                res.lower(import_path.clone(), &mut program, &mut imports, &mut output);
                imported.insert(i);
            }
        }
        (program, output)
    }
}

fn run_file(path: &Path, gdb: bool, asm: bool) {
    let (mut program, mut output) = UProgram::from_path(path);
    program.resolve(&mut output);
    // println!("vars:");
    // for (id, def) in program.iter_vars() {
    //     println!("    {id:?} = {}: {}", program.names.path(id), program.type_name(&def.ty));
    // }
    // for (id, f) in program.iter_fns() {
    //     println!("{}:{id:?} = {:#?}", program.names.path(id), f);
    // }
    if !output.errs.is_empty() {
        output.write_to(&mut stdout());
        return;
    }
    let program = LProgram::create(&program).expect("morir");
    let unlinked = compiler::compile(&program);
    if asm {
        println!("{:?}", unlinked);
    } else {
        let bin = unlinked.link().to_elf();
        println!("compiled");
        save_run(&bin, gdb);
    }
    output.write_to(&mut stdout());
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
    println!("todo");
    // for line in BufReader::new(std::io::stdin()).lines() {
    //     let str = &line.expect("failed to read line");
    //     let mut ctx = ParserCtx::from(&str[..]);
    //     if let Some(expr) = PStatement::parse_node(&mut ctx).node.as_ref() {
    //         if ctx.next().is_none() {
    //             println!("{:?}", expr);
    //         } else {
    //             println!("uhhhh ehehe");
    //         }
    //     }
    //     ctx.output.write_for(&mut stdout(), str);
    // }
}
