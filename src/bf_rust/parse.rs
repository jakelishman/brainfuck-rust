use structure::*;

pub fn to_base_ops (input: &String) -> Vec<BaseOp> {
    input.chars()
        .filter_map(
            |op|
            match op {
                '+' => Some(BaseOp::IncrementData),
                '-' => Some(BaseOp::DecrementData),
                '>' => Some(BaseOp::IncrementPointer),
                '<' => Some(BaseOp::DecrementPointer),
                '[' => Some(BaseOp::StartLoop),
                ']' => Some(BaseOp::EndLoop),
                ',' => Some(BaseOp::Read),
                '.' => Some(BaseOp::Write),
                _   => None,
            })
        .collect()
}

pub fn find_matching_end_loop (input: &Vec<BaseOp>,
                           start_pos: usize) -> Option<usize> {
    let mut open_loops = 0;
    for i in start_pos + 1 .. input.len() {
        match input[i] {
            BaseOp::EndLoop if open_loops == 0 => return Some(i),
            BaseOp::EndLoop   => open_loops -= 1,
            BaseOp::StartLoop => open_loops += 1,
            _                 => (),
        }
    }
    None
}

fn to_ops_helper (input: &Vec<BaseOp>,
                  start_pos: usize,
                  end_pos: usize) -> Result<Vec<Expression>, String> {
    let mut out = vec![];
    let mut i = start_pos;
    while i < end_pos {
        let expr = match input[i] {
            BaseOp::IncrementData => {
                i += 1;
                Expression::Op (Op::ChangeData (1)) },
            BaseOp::DecrementData => {
                i += 1;
                Expression::Op (Op::ChangeData (-1)) },
            BaseOp::IncrementPointer => {
                i += 1;
                Expression::Op (Op::ChangePointer (1)) },
            BaseOp::DecrementPointer => {
                i += 1;
                Expression::Op (Op::ChangePointer (-1)) },
            BaseOp::Read => {
                i += 1;
                Expression::Op (Op::Read (1)) },
            BaseOp::Write => {
                i += 1;
                Expression::Op (Op::Write (1)) },
            BaseOp::EndLoop => {
                return Err(format!("Unexpected ']' in position {}.", i));
            },
            BaseOp::StartLoop => {
                let start = i;
                i = match find_matching_end_loop(input, start) {
                    Some(end) => end + 1,
                    None      => {
                        let msg =
                            format!("Unterminated '[' at position {}.", start);
                        return Err(msg);
                    },
                };
                Expression::Loop (
                    Box::new(to_ops_helper(input, start + 1, i - 1)?),
                    None
                )
            },
        };
        out.push(expr);
    };

    Ok (out)
}

pub fn to_ops (input: &Vec<BaseOp>) -> Result<Vec<Expression>, String> {
    to_ops_helper(input, 0, input.len())
}
