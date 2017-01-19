use std::io;
use std::io::{Read, Write};

fn interpret (instructions: &Vec<u8>, input: &mut Read, output: &mut Write) {
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
                format!("Increment data pointer to {}.", dptr) },
            '<' => {
                dptr -= 1;
                iptr += 1;
                format!("Decrement data pointer to {}.", dptr) },
            '+' => {
                data[dptr] = match data[dptr] { 255 => 0, num => num + 1, };
                iptr += 1;
                format!("Increment value at {} to {}.", dptr, data[dptr]) },
            '-' => {
                data[dptr] = match data[dptr] { 0 => 255, num => num - 1, };
                iptr += 1;
                format!("Decrement value at {} to {}.", dptr, data[dptr]) },
            '.' => {
                write!(output, "{}", data[dptr] as char).expect("Failed to write.");
                iptr += 1;
                format!("Output character '{}'.", data[dptr] as char) },
            ',' => {
                data[dptr] =
                    match input.read(&mut buf) {
                        Ok(1) => buf[0],
                        _     => data[dptr],
                    };
                iptr += 1;
                format!("Read character from input '{}'.", data[dptr] as char) },
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
                    format!("Finished loop with data pointer {}.", dptr)
                } else {
                    loop_points.push(iptr);
                    iptr += 1;
                    format!("Loop with data[{}] = {}.", dptr, data[dptr])
                } },
            ']' => {
                iptr = loop_points.pop().expect("Mismatched loop points.");
                format!("Returning to beginning of loop at {}.", iptr) },
            c   => {
                iptr += 1;
                format!("Skipping non-control character '{}'.", c) },
        };
    }
}

fn read_instructions (file_path: &String) -> Result<Vec<u8>, io::Error> {
    use std::fs::File;
    let mut instructions = vec![];
    File::open(file_path)?.read_to_end(&mut instructions)?;
    Ok(instructions)
}

struct CommandLineFlags {
    debug: bool,
    version: bool,
    help: bool,
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
            unknown => writeln!(io::stderr(), "Unknown command line flag: '{}'.", unknown)
                           .expect("Failed to write to stderr."),
        }
    }
    (others, flags)
}

fn show_version() {
    writeln!(io::stderr(), "brainfuck_rust interpreter: version 0.1.0")
        .expect("Failed to write to stderr.");
}

fn show_help() {
    writeln!(io::stderr(), "\
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
      Show the version information.")
        .expect("Failed to write to stderr.");
}

fn main () {
    let mut stdin  = io::stdin();
    let mut stdout = io::stdout();
    stdin.lock();
    stdout.lock();

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
            Ok(ins)  => interpret(&ins, &mut stdin, &mut stdout),
            Err(err) =>
                writeln!(io::stderr(), "Failed to open file '{}' with error:\n{}", file, err)
                    .expect("Failed to write to stderr."),
        };
    }
}
