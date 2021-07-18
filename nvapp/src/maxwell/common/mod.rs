#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum ReductionOperation {
    Add,
    Min,
    Max,
    Increment,
    Decrement,
    And,
    Or,
    Xor,
    Unknown(u32),
}

impl From<ReductionOperation> for u32 {
    fn from(mode: ReductionOperation) -> u32 {
        match mode {
            ReductionOperation::Add => 0,
            ReductionOperation::Min => 1,
            ReductionOperation::Max => 2,
            ReductionOperation::Increment => 3,
            ReductionOperation::Decrement => 4,
            ReductionOperation::And => 5,
            ReductionOperation::Or => 6,
            ReductionOperation::Xor => 7,
            ReductionOperation::Unknown(val) => val,
        }
    }
}

impl From<u32> for ReductionOperation {
    fn from(mode: u32) -> ReductionOperation {
        match mode {
            0 => ReductionOperation::Add,
            1 => ReductionOperation::Min,
            2 => ReductionOperation::Max,
            3 => ReductionOperation::Increment,
            4 => ReductionOperation::Decrement,
            5 => ReductionOperation::And,
            6 => ReductionOperation::Or,
            7 => ReductionOperation::Xor,
            val => ReductionOperation::Unknown(val),
        }
    }
}