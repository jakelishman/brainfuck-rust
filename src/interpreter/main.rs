use std::io;
use std::io::{Read, Write};

extern crate bf_rust;
use bf_rust::structure::*;
use bf_rust::parse;

macro_rules! debug_message {
    ($flag: expr, $dst: expr, $fmt: expr) => {
        if $flag {
            writeln!($dst, $fmt).expect("Failed to write debug message.");
        }
    };
    ($flag: expr, $dst: expr, $fmt: expr, $($arg: expr),+) => {
        if $flag {
            writeln!($dst, $fmt, $($arg),+).expect("Failed to write debug message.")
        }
    };
}

struct CommandLineFlags {
    debug: bool,
    native: bool,
    version: bool,
    help: bool,
}

struct StreamSet<'a> {
    in_: &'a mut Read,
    out: &'a mut Write,
    err: &'a mut Write,
}

fn interpret_base<'a> (instructions: &Vec<BaseOp>,
                       streams: &'a mut StreamSet,
                       flags: &CommandLineFlags) {
    let mut loop_points = Vec::new();
    let mut data = [0u8; 1024];
    let mut iptr: usize = 0;
    let mut dptr: usize = 0;

    let mut buf = [0u8];

    while iptr < instructions.len() {
        match instructions[iptr] {
            BaseOp::IncrementPointer => {
                dptr += 1;
                iptr += 1;
                debug_message!(flags.debug, streams.err,
                               "Increment data pointer to {}.", dptr) },
            BaseOp::DecrementPointer => {
                dptr -= 1;
                iptr += 1;
                debug_message!(flags.debug, streams.err,
                               "Decrement data pointer to {}.", dptr) },
            BaseOp::IncrementData => {
                data[dptr] = match data[dptr] { 255 => 0, num => num + 1, };
                iptr += 1;
                debug_message!(flags.debug, streams.err,
                               "Increment value at {} to {}.", dptr, data[dptr]) },
            BaseOp::DecrementData => {
                data[dptr] = match data[dptr] { 0 => 255, num => num - 1, };
                iptr += 1;
                debug_message!(flags.debug, streams.err,
                               "Decrement value at {} to {}.", dptr, data[dptr]) },
            BaseOp::Write => {
                write!(streams.out, "{}", data[dptr] as char)
                    .expect("Failed to write.");
                iptr += 1;
                debug_message!(flags.debug, streams.err,
                               "Output character '{}'.", data[dptr] as char) },
            BaseOp::Read => {
                data[dptr] =
                    match streams.in_.read(&mut buf) {
                        Ok(1) => buf[0],
                        _     => 0,
                    };
                iptr += 1;
                debug_message!(flags.debug, streams.err,
                               "Read character from input '{}'.",
                               data[dptr] as char) },
            BaseOp::StartLoop => {
                if data[dptr] == 0u8 {
                    iptr = parse::find_matching_end_loop(instructions, iptr)
                               .expect("Unterminated loop.") + 1;
                    debug_message!(flags.debug, streams.err,
                                   "Finished loop with data pointer {}.", dptr)
                } else {
                    loop_points.push(iptr);
                    iptr += 1;
                    debug_message!(flags.debug, streams.err,
                                   "Loop with data[{}] = {}.", dptr, data[dptr])
                } },
            BaseOp::EndLoop => {
                iptr = loop_points.pop().expect("Mismatched loop points.");
                debug_message!(flags.debug, streams.err,
                               "Returning to beginning of loop at {}.", iptr) },
        };
    }
}

fn interpret_native_helper<'a> (instructions: &Vec<Expression>,
                                streams: &'a mut StreamSet,
                                flags: &CommandLineFlags,
                                data: &mut Vec<u8>,
                                dptr: &mut usize) {
    use bf_rust::structure::Expression::*;
    use bf_rust::structure::Op::*;

    for expr in instructions.iter() {
        match expr {
            &Loop(ref body, None)     => {
                let mut i = 0;
                debug_message!(flags.debug, streams.err,
                               "Entering loop of unknown length.");
                while data[*dptr] != 0 {
                    debug_message!(flags.debug, streams.err,
                                   "Iterating loop of unknown length, count {}.",
                                   i);
                    i += 1;
                    interpret_native_helper(&body, streams, flags, data, dptr);
                }
                debug_message!(flags.debug, streams.err,
                               "Finished loop of unknown length, iterations: {}.",
                               i);
            },
            &Loop(ref body, Some(i))  => {
                debug_message!(flags.debug, streams.err,
                               "Entering loop of length {}.", i);
                for j in 0 .. i {
                    debug_message!(flags.debug, streams.err,
                                   "Iterating loop count {}.", j);
                    interpret_native_helper(&body, streams, flags, data, dptr);
                }
                debug_message!(flags.debug, streams.err,
                               "Finished loop of length {}.", i);
            },
            &Op(ChangeData(i))    => {
                data[*dptr] = ((data[*dptr] as i16 + i as i16 + 256) % 256) as u8;
                debug_message!(flags.debug, streams.err,
                               "Changing data at {} by {} to {}.", *dptr, i,
                               data[*dptr]);
            }
            &Op(ChangePointer(i)) => {
                *dptr = (*dptr as i64 + i as i64) as usize;
                debug_message!(flags.debug, streams.err,
                              "Moving data pointer by {} to {}.", i, *dptr);
            }
            &Op(Read(len))        => {
                let mut buf = Vec::with_capacity(len);
                unsafe { buf.set_len(len) };
                let _ = streams.in_.read(buf.as_mut_slice());
                if data.len() < *dptr + len {
                    data.reserve(len);
                }
                for i in 0 .. len {
                    data[*dptr + i] = buf[i];
                }
                debug_message!(flags.debug, streams.err,
                               "Read {} bytes from input: '{}'.", len,
                               buf.iter().map(|&c| c as char).collect::<String>());
                *dptr += len - 1;
            },
            &Op(Write(len))       => {
                for i in 0 .. len {
                    print!("{}", data[*dptr + i] as char);
                }
                debug_message!(flags.debug, streams.err,
                               "Write {} bytes.", len);
                *dptr += len - 1;
            },
        }
    }
}


fn interpret_native<'a> (instructions: &Vec<Expression>,
                         streams: &'a mut StreamSet,
                         flags: &CommandLineFlags) {
    let mut data = vec![0u8; 2048];
    let mut dptr = 0;
    interpret_native_helper(instructions, streams, flags, &mut data, &mut dptr);
}

fn interpret<'a> (instructions: &Program,
                  streams: &'a mut StreamSet,
                  flags: &CommandLineFlags) {
    match *instructions {
        Program::Base(ref base_ops)     => interpret_base(base_ops, streams, flags),
        Program::Native(ref native_ops) => interpret_native(native_ops, streams, flags),
    }
}

fn read_instructions (file_path: &String) ->
Result<String, io::Error> {
    use std::fs::File;
    let mut instructions = String::new();
    File::open(file_path)?.read_to_string(&mut instructions)?;
    Ok(instructions)
}

fn parse_command_line_flags (args: std::env::Args) -> (Vec<String>, CommandLineFlags) {
    let mut others = vec![];
    let mut individual_flags = vec![];
    let mut flags = CommandLineFlags {
        debug: false, native: false, version: false, help: false };
    for arg in args.skip(1) {
        if arg.starts_with("--") {
            individual_flags.push(arg[2..].to_string());
        } else if arg.starts_with("-") {
            for c in arg.chars().skip(1) {
                individual_flags.push(c.to_string())
            }
        } else {
            others.push(arg);
        }
    }

    for flag in individual_flags.iter() {
        match flag.as_ref() {
            "d" | "debug" => flags.debug = true,
            "n" | "native" => flags.native = true,
            "h" | "help"  => flags.help = true,
            "v" | "version" => flags.version = true,
            unknown => debug_message!(true, io::stderr(), "Unknown command line flag: '{}'.", unknown)
        }
    }
    (others, flags)
}

fn show_version() {
    debug_message!(true, io::stderr(), "brainfuck_rust interpreter: version 0.1.0")
}

fn show_help() {
    debug_message!(true, io::stderr(), "\
SYNOPSIS
  interpreter [-dn] file1.bf [file2.bf ...]
  interpreter -h
  interpreter -v

OPTIONS
  -d --debug
      Enable debug messages for each brainfuck command parsed.
  -n --native
      Compiles the base opcodes into the native internal representation, without
      running the optimiser.
  -h --help
      Print this help message and exit.
  -v --version
      Show the version information.");
}

fn compile (string: &String, flags: &CommandLineFlags)
-> Result<Program, String> {
    let base = parse::to_base_ops(string);
    if flags.native {
        parse::to_ops(&base).map(Program::Native)
    } else {
        Ok(Program::Base(base))
    }
}

fn main () {
    let mut stdin  = io::stdin();
    let mut stdout = io::stdout();
    let mut stderr = io::stderr();
    stdin.lock();
    stdout.lock();
    stderr.lock();

    let (files, flags) = parse_command_line_flags(std::env::args());

    if flags.version {
        show_version();
        return;
    }

    if flags.help || files.len() == 0 {
        show_help();
        return;
    }

    for file in files.iter() {
        match read_instructions(&file) {
            Ok(contents)  => {
                let mut streams = StreamSet {
                    in_: &mut stdin,
                    out: &mut stdout,
                    err: &mut stderr,
                };
                match compile(&contents, &flags) {
                    Ok(prog) => interpret(&prog, &mut streams, &flags),
                    Err(s)   => debug_message!(true, io::stderr(),
                                "Invalid brainfuck program '{}':\n{}", file, s),
                };
            },
            Err(err) => {
                debug_message!(true, io::stderr(),
                "Failed to open file '{}' with error:\n{}", file, err);
            },
        };
    }
}
