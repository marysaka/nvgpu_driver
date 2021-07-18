#![recursion_limit = "1024"]
#![allow(dead_code)]

use nvgpu::NvGpuResult;

#[macro_use]
extern crate bitfield;

mod maxwell;
mod utils;

use crate::utils::{Command, CommandStream, CommandSubmissionMode, SubChannelId};
use maxwell::common::*;
use maxwell::compute::*;
use maxwell::dma::*;
use maxwell::threed::*;
use utils::GpuBox;


fn main() -> NvGpuResult<()> {
    let nvgpu_channel = utils::initialize().unwrap();

    let mut command_stream = utils::initialize_command_stream(&nvgpu_channel)?;

    let query_res_buffer = GpuBox::new([0x0u64; 2]);
    let copy_res_buffer = GpuBox::new([0x0u64; 2]);

    let mut report_control = ReportControl::new();
    report_control.set_operation(ReportControlOperation::Counter);
    report_control.set_one_word(false);
    report_control.set_fence_enable(false);
    report_control.set_flush_disable(false);
    report_control.set_reduction_enable(false);
    report_control.set_counter_type(ReportCounterType::SamplesPassed);
    report_control.set_reduction_operation(ReductionOperation::Add);

    println!("query_res_buffer: {:?}", &query_res_buffer[..]);
    query_get(
        &mut command_stream,
        query_res_buffer.gpu_address(),
        0,
        report_control,
    )?;

    memcpy_1d(
        &mut command_stream,
        copy_res_buffer.gpu_address(),
        query_res_buffer.gpu_address(),
        query_res_buffer.user_size() as u32,
    )?;

    memcpy_inline_host_to_device(
        &mut command_stream,
        copy_res_buffer.gpu_address(),
        &[42],
    )?;

    // Send the commands to the GPU.
    command_stream.flush()?;

    // Wait for the operations to be complete on the GPU side.
    command_stream.wait_idle();

    println!("query_res_buffer: {:?}", &query_res_buffer[..]);
    println!("copy_res_buffer: {:?}", &copy_res_buffer[..]);

    let mut qmd = QueueMetaData17([0x0; 0x40]);

    qmd.set_dependent_qmd_pointer(0x42);

    let mut release = QueueMetaData17Release([0x0; 0x3]);

    release.set_payload(u32::MAX);

    qmd.set_release(0, &release);
    qmd.set_release(1, &release);

    let const_buffer = QueueMetaData17ConstantBuffer(0x0);

    qmd.set_constant_buffer(0, &const_buffer);

    println!("qmd[6]: {}", &qmd.0[0x17 + 2]);
    println!("qmd: {:?}", &qmd.0[..]);


    Ok(())
}
