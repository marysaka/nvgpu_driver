use crate::utils::{Command, CommandStream, CommandSubmissionMode, SubChannelId};
use nvgpu::{GpuVirtualAddress, NvGpuResult};

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum ReportCounterType {
    Zero,
    InputVertices,
    InputPrimitives,
    VertexShaderInvocations,
    GeometryShaderInvocations,
    GeometryShaderPrimitives,
    TransformFeedbackPrimitivesWritten,
    ClipperInputPrimitives,
    ClipperOutputPrimitives,
    PrimitivesGenerated,
    FragmentShaderInvocations,
    SamplesPassed,
    TessControlShaderInvocations,
    TessEvaluationShaderInvocations,
    TessEvaluationShaderPrimitives,
    ZcullStats0,
    ZcullStats1,
    ZcullStats2,
    ZcullStats3,
    Unknown(u32),
}

impl From<ReportCounterType> for u32 {
    fn from(mode: ReportCounterType) -> u32 {
        match mode {
            ReportCounterType::Zero => 0,
            ReportCounterType::InputVertices => 1,
            ReportCounterType::InputPrimitives => 3,
            ReportCounterType::VertexShaderInvocations => 5,
            ReportCounterType::GeometryShaderInvocations => 5,
            ReportCounterType::GeometryShaderPrimitives => 5,
            ReportCounterType::TransformFeedbackPrimitivesWritten => 0xb,
            ReportCounterType::ClipperInputPrimitives => 0xf,
            ReportCounterType::ClipperOutputPrimitives => 0x11,
            ReportCounterType::PrimitivesGenerated => 0x12,
            ReportCounterType::FragmentShaderInvocations => 0x13,
            ReportCounterType::SamplesPassed => 0x15,
            ReportCounterType::TessControlShaderInvocations => 0x1b,
            ReportCounterType::TessEvaluationShaderInvocations => 0x1d,
            ReportCounterType::TessEvaluationShaderPrimitives => 0x1f,
            ReportCounterType::ZcullStats0 => 0x2a,
            ReportCounterType::ZcullStats1 => 0x2c,
            ReportCounterType::ZcullStats2 => 0x2e,
            ReportCounterType::ZcullStats3 => 0x30,
            ReportCounterType::Unknown(val) => val,
        }
    }
}

impl From<u32> for ReportCounterType {
    fn from(mode: u32) -> ReportCounterType {
        match mode {
            0 => ReportCounterType::Zero,
            1 => ReportCounterType::InputVertices,
            3 => ReportCounterType::InputPrimitives,
            5 => ReportCounterType::VertexShaderInvocations,
            7 => ReportCounterType::GeometryShaderInvocations,
            9 => ReportCounterType::GeometryShaderPrimitives,
            0xb => ReportCounterType::TransformFeedbackPrimitivesWritten,
            0xf => ReportCounterType::ClipperInputPrimitives,
            0x11 => ReportCounterType::ClipperOutputPrimitives,
            0x12 => ReportCounterType::PrimitivesGenerated,
            0x13 => ReportCounterType::FragmentShaderInvocations,
            0x15 => ReportCounterType::SamplesPassed,
            0x1b => ReportCounterType::TessControlShaderInvocations,
            0x1d => ReportCounterType::TessEvaluationShaderInvocations,
            0x1f => ReportCounterType::TessEvaluationShaderPrimitives,
            0x2a => ReportCounterType::ZcullStats0,
            0x2c => ReportCounterType::ZcullStats1,
            0x2e => ReportCounterType::ZcullStats2,
            0x30 => ReportCounterType::ZcullStats3,
            val => ReportCounterType::Unknown(val),
        }
    }
}

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

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum ReportControlOperation {
    Release,
    Counter,
    Trap,
    Unknown(u32),
}

impl From<ReportControlOperation> for u32 {
    fn from(mode: ReportControlOperation) -> u32 {
        match mode {
            ReportControlOperation::Release => 0,
            ReportControlOperation::Counter => 2,
            ReportControlOperation::Trap => 3,
            ReportControlOperation::Unknown(val) => val,
        }
    }
}

impl From<u32> for ReportControlOperation {
    fn from(mode: u32) -> ReportControlOperation {
        match mode {
            0 => ReportControlOperation::Release,
            2 => ReportControlOperation::Counter,
            3 => ReportControlOperation::Trap,
            val => ReportControlOperation::Unknown(val),
        }
    }
}

bitfield! {
    pub struct ReportControl(u32);
    impl Debug;

    #[inline]
    pub from into ReportControlOperation, operation, set_operation: 1, 0;

    #[inline]
    pub flush_disable, set_flush_disable: 2;

    #[inline]
    pub reduction_enable, set_reduction_enable: 3;

    // ???
    #[inline]
    pub fence_enable, set_fence_enable: 4;

    #[inline]
    pub from into ReductionOperation, reduction_operation, set_reduction_operation: 11, 9;

    // NOTE: All bits need to be set.
    #[inline]
    pub reserved, set_reserved: 15, 12;

    #[inline]
    // TODO: enum this
    pub reduction_signed, set_reduction_signed: 17;

    #[inline]
    pub from into ReportCounterType, counter_type, set_counter_type: 27, 23;

    #[inline]
    pub is_one_word, set_one_word: 28;
}

impl ReportControl {
    pub fn new() -> ReportControl {
        let mut result = ReportControl(0);

        result.set_reserved(0xF);

        result
    }
}

pub fn query_get(
    command_stream: &mut CommandStream,
    gpu_va: GpuVirtualAddress,
    payload: u32,
    report_control: ReportControl,
) -> NvGpuResult<()> {
    let mut query_get = Command::new(
        0x6c0,
        SubChannelId::ThreeD,
        CommandSubmissionMode::Increasing,
    );

    query_get.push_address(gpu_va);
    query_get.push_argument(payload);
    query_get.push_argument(report_control.0);

    // Push the command
    command_stream.push(query_get)
}
