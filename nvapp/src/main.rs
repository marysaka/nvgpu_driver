use nvgpu::{enum_with_val, NvGpuResult, GpuVirtualAddress};

#[macro_use]
extern crate bitfield;

mod utils;

use utils::{GpuBox, Command, CommandStream, CommandSubmissionMode, SubChannelId};

enum_with_val! {
    #[derive(PartialEq, Eq, Clone, Copy)]
    pub struct ReportCounterType(pub u32) {
        ZERO = 0,
        INPUT_VERTICES = 1,
        INPUT_PRIMITIVES = 3,
        VERTEX_SHADER_INVOCATIONS = 5,
        GEOMETRY_SHADER_INVOCATIONS = 7,
        GEOMETRY_SHADER_PRIMITIVES = 9,
        TRANSFORM_FEEDBACK_PRIMITIVES_WRITTEN = 0xb,
        CLIPPER_INPUT_PRIMITIVES = 0xf,
        CLIPPER_OUTPUT_PRIMITIVES = 0x11,
        PRIMITIVES_GENERATED = 0x12,
        FRAGMENT_SHADER_INVOCATIONS = 0x13,
        SAMPLES_PASSED = 0x15,
        TESS_CONTROL_SHADER_INVOCATIONS = 0x1b,
        TESS_EVALUATION_SHADER_INVOCATIONS = 0x1d,
        TESS_EVALUATION_SHADER_PRIMITIVES = 0x1f,
        ZUL_STATS_0 = 0x2a,
        ZUL_STATS_1 = 0x2c,
        ZUL_STATS_2 = 0x2e,
        ZUL_STATS_3 = 0x30
    }
}


#[derive(Debug, Clone, Copy)]
pub enum ReductionOperation {
    ADD,
    MIN,
    MAX,
    INCREMENT,
    DECREMENT,
    AND,
    OR,
    XOR,
    Unknown(u32)
}


impl From<ReductionOperation> for u32 {
    fn from(mode: ReductionOperation) -> u32 {
        match mode {
            ReductionOperation::ADD => 0,
            ReductionOperation::MIN => 1,
            ReductionOperation::MAX => 2,
            ReductionOperation::INCREMENT => 3,
            ReductionOperation::DECREMENT => 4,
            ReductionOperation::AND => 5,
            ReductionOperation::OR => 6,
            ReductionOperation::XOR => 7,
            ReductionOperation::Unknown(val) => val,
        }
    }
}

impl From<u32> for ReductionOperation {
    fn from(mode: u32) -> ReductionOperation {
        match mode {
            0 => ReductionOperation::ADD,
            1 => ReductionOperation::MIN,
            2 => ReductionOperation::MAX,
            3 => ReductionOperation::INCREMENT,
            4 => ReductionOperation::DECREMENT,
            5 => ReductionOperation::AND,
            6 => ReductionOperation::OR,
            7 => ReductionOperation::XOR,
            val => ReductionOperation::Unknown(val),
        }
    }
}

bitfield! {
    pub struct ReportControl(u32);
    impl Debug;

    #[inline]
    pub operation, set_operation: 1, 0;

    #[inline]
    pub flush_disable, set_flush_disable: 2;

    #[inline]
    pub reduction_enable, set_reduction_enable: 3;

    // ???
    #[inline]
    pub fence_enable, set_fence_enable: 4;

    pub reduction_operation, set_reduction_operation: 11, 9;

    #[inline]
    pub counter_type, set_counter_type: 27, 23;

    #[inline]
    pub is_one_word, set_one_word: 28;
}

enum_with_val! {
    #[derive(PartialEq, Eq, Clone, Copy)]
    pub struct ReportControlOperation(pub u32) {
        RELEASE = 0,
        COUNTER = 2,
        TRAP = 3
    }
}

pub fn query_get(command_stream: &mut CommandStream, gpu_va: GpuVirtualAddress, payload: u32, report_control: ReportControl) -> NvGpuResult<()> {
    let mut query_get = Command::new(0x6c0, SubChannelId::THREE_D, CommandSubmissionMode::Increasing);

    query_get.push_address(gpu_va);
    query_get.push_argument(payload);
    query_get.push_argument(report_control.0);

    // Push the command
    command_stream.push(query_get)
}

fn main() -> NvGpuResult<()> {
    let nvgpu_channel = utils::initialize().unwrap();

    let mut command_stream = utils::initialize_command_stream(&nvgpu_channel)?;

    let payload = 0x0;

    let query_res_buffer = GpuBox::new([0x0u64; 2]);

    let mut report_control = ReportControl(0);

    report_control.set_operation(ReportControlOperation::COUNTER.0);
    report_control.set_one_word(false);
    report_control.set_fence_enable(false);
    report_control.set_flush_disable(false);
    report_control.set_reduction_enable(false);
    report_control.set_counter_type(ReportCounterType::SAMPLES_PASSED.0);
    report_control.set_reduction_operation(u32::from(ReductionOperation::INCREMENT));
    println!("{:?}", report_control);
    println!("{:?}", ReportControl(0xa80f002));

    println!("before: {:?}", &query_res_buffer[..]);
    query_get(&mut command_stream, query_res_buffer.gpu_address(), payload, report_control)?;

    // Send the commands to the GPU.
    command_stream.flush()?;

    // Wait for the operations to be complete on the GPU side.
    command_stream.wait_idle();

    println!("after: {:?}", &query_res_buffer[..]);

    Ok(())
}
