use std::io;
use std::io::{Read, Write};

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
    version: bool,
    help: bool,
}

struct StreamSet<'a> {
    in_: &'a mut Read,
    out: &'a mut Write,
    err: &'a mut Write,
}

fn interpret<'a> (instructions: &Vec<u8>, streams: &'a mut StreamSet, flags: &CommandLineFlags) {
    let mut loop_points = Vec::new();
    let mut data = [0u8; 1024];
    let mut iptr = 0;
    let mut dptr = 0;

    let mut buf = [0u8];

    while iptr < instructions.len() {
        match instructions[iptr] as char {
            '>' => {
                dptr += 1;
                iptr += 1;
                debug_message!(flags.debug, streams.err,
                               "Increment data pointer to {}.", dptr) },
            '<' => {
                dptr -= 1;
                iptr += 1;
                debug_message!(flags.debug, streams.err,
                               "Decrement data pointer to {}.", dptr) },
            '+' => {
                data[dptr] = match data[dptr] { 255 => 0, num => num + 1, };
                iptr += 1;
                debug_message!(flags.debug, streams.err,
                               "Increment value at {} to {}.", dptr, data[dptr]) },
            '-' => {
                data[dptr] = match data[dptr] { 0 => 255, num => num - 1, };
                iptr += 1;
                debug_message!(flags.debug, streams.err,
                               "Decrement value at {} to {}.", dptr, data[dptr]) },
            '.' => {
                write!(streams.out, "{}", data[dptr] as char).expect("Failed to write.");
                iptr += 1;
                debug_message!(flags.debug, streams.err,
                               "Output character '{}'.", data[dptr] as char) },
            ',' => {
                data[dptr] =
                    match streams.in_.read(&mut buf) {
                        Ok(1) => buf[0],
                        _     => 0,
                    };
                iptr += 1;
                debug_message!(flags.debug, streams.err,
                               "Read character from input '{}'.", data[dptr] as char) },
            '[' => {
                if data[dptr] == 0u8 {
                    let mut open_loops = 0;
                    loop {
                        iptr += 1;
                        if iptr == instructions.len() { return; }
                        match instructions[iptr] as char {
                            '[' => { open_loops += 1; },
                            ']' => { if open_loops == 0 { break; } else { open_loops -= 1; } },
                            _   => (),
                        }
                    }
                    iptr += 1;
                    debug_message!(flags.debug, streams.err,
                                   "Finished loop with data pointer {}.", dptr)
                } else {
                    loop_points.push(iptr);
                    iptr += 1;
                    debug_message!(flags.debug, streams.err,
                                   "Loop with data[{}] = {}.", dptr, data[dptr])
                } },
            ']' => {
                iptr = loop_points.pop().expect("Mismatched loop points.");
                debug_message!(flags.debug, streams.err,
                               "Returning to beginning of loop at {}.", iptr) },
            c   => {
                iptr += 1;
                debug_message!(flags.debug, streams.err,
                               "Skipping non-control character '{}'.", c) },
        };
    }
}

fn read_instructions (file_path: &String) -> Result<Vec<u8>, io::Error> {
    use std::fs::File;
    let mut instructions = vec![];
    File::open(file_path)?.read_to_end(&mut instructions)?;
    Ok(instructions)
}

fn parse_command_line_flags (args: std::env::Args) -> (Vec<String>, CommandLineFlags) {
    let mut others = vec![];
    let mut individual_flags = vec![];
    let mut flags = CommandLineFlags { debug: false, version: false, help: false };
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
  interpreter [-d] file1.bf [file2.bf ...]
  interpreter -h
  interpreter -v

OPTIONS
  -d --debug
      Enable debug messages for each brainfuck command parsed.
  -h --help
      Print this help message and exit.
  -v --version
      Show the version information.");
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
            Ok(ins)  => interpret(&ins, &mut StreamSet { in_: &mut stdin, out: &mut stdout, err: &mut stderr }, &flags),
            Err(err) => debug_message!(true, io::stderr(), "Failed to open file '{}' with error:\n{}", file, err)
        };
    }
}
