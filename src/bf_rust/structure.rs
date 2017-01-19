#[derive(Debug)]
pub enum BaseOp {
    IncrementData,
    DecrementData,
    IncrementPointer,
    DecrementPointer,
    StartLoop,
    EndLoop,
    Read,
    Write,
}

#[derive(Debug)]
pub enum Op {
    ChangeData    (i16),
    ChangePointer (i16),
    Read          (usize),
    Write         (usize),
}

#[derive(Debug)]
pub enum Expression {
    Loop (Box<Vec<Expression>>, Option<usize>),
    Op   (Op),
}

#[derive(Debug)]
pub enum Program {
    Base   (Box<Vec<BaseOp>>),
    Native (Expression),
}
