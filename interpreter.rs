use std::io;
use std::io::{Read, Write};

fn interpret (instructions: &Vec<u8>) {
    let mut loop_points = Vec::new();
    let mut data = [0u8; 1024];
    let mut iptr = 0;
    let mut dptr = 0;

    let mut stdio = io::stdin();
    stdio.lock();

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
                data[dptr] += 1;
                iptr += 1;
                format!("Increment value at {} to {}.", dptr, data[dptr]) },
            '-' => {
                data[dptr] -= 1;
                iptr += 1;
                format!("Decrement value at {} to {}.", dptr, data[dptr]) },
            '.' => {
                print!("{}", data[dptr] as char);
                iptr += 1;
                format!("Output character '{}'.", data[dptr] as char) },
            '[' => {
                if data[dptr] == 0u8 {
                    let mut open_loops = 0;
                    loop {
                        iptr += 1;
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
