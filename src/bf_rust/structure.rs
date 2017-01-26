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
    ChangeData    (i8),
    ChangePointer (i16),
    Read          (usize),
    Write         (usize),
}

#[derive(Debug)]
pub enum Expression {
    Loop (Vec<Expression>, Option<usize>),
    Op   (Op),
}

#[derive(Debug)]
pub enum Program {
    Base   (Vec<BaseOp>),
    Native (Vec<Expression>),
}
