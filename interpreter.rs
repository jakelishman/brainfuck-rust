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

fn main () {
    let mut stderr = io::stderr();
    let mut stdin  = io::stdin();
    let mut stdout = io::stdout();
    stdin.lock();
    stdout.lock();

    let args = std::env::args();
    if args.len() == 1 {
        writeln!(stderr, "I need a list of input files as arguments.")
            .expect("Failed to write to stderr.");
        std::process::exit(1);
    }

    for argument in args.skip(1) {
        match read_instructions(&argument) {
            Ok(ins)  => interpret(&ins, &mut stdin, &mut stdout),
            Err(err) =>
                writeln!(stderr, "Failed to open file '{}' with error:\n{}", argument, err)
                    .expect("Failed to write to stderr."),
        };
        println!("");
    }
}
