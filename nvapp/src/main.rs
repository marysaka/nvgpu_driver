#![recursion_limit = "1024"]
#![allow(dead_code)]

// TODO: arch dependent code (use nvgpu_gpu_get_characteristics)
// TODO: grab wrap count, sm count and memory size.
use nvgpu::NvGpuResult;

#[macro_use]
extern crate bitfield;

mod maxwell;
mod utils;

use maxwell::compute::*;
use maxwell::dma::*;
use utils::{align_up, GpuAllocated, GpuBox};

use nvgpu::GpuCharacteristics;

const PROGRAM_REGION_ALIGNMENT: usize = 0x1000000;
const SCRATCH_MEMORY_ALIGNMENT: usize = 0x20000;
const DEFAULT_SCRATCH_MEMORY_PER_SM: usize = 0x800;
// TODO: define bindless texture constant buffer layout
const BINDLESS_TEXTURE_CBUFF_INDEX: u32 = 0;

fn compute_total_scratch_size(
    gpu_characteristics: &GpuCharacteristics,
    wrap_scratch_size: u32,
) -> u32 {
    align_up(
        wrap_scratch_size
            * gpu_characteristics.sm_arch_warp_count
            * gpu_characteristics.num_gpc
            * gpu_characteristics.num_tpc_per_gpc,
        SCRATCH_MEMORY_ALIGNMENT as u32,
    )
}

fn main() -> NvGpuResult<()> {
    let (gpu_channel, gpu_characteristics) = utils::initialize().unwrap();

    assert_eq!(gpu_characteristics.chip_name(), "gm20b");

    let mut command_stream = utils::initialize_command_stream(&gpu_channel)?;

    println!("{:?}", gpu_characteristics);
    println!(
        "Running on chip named {:?}",
        gpu_characteristics.chip_name()
    );

    // TODO: fancy address space allocation (one day)
    let program_region = GpuBox::new_with_alignment([0xAAAAAAAAu64; 1], PROGRAM_REGION_ALIGNMENT);
    let scratch_memory = GpuAllocated::new(
        compute_total_scratch_size(&gpu_characteristics, DEFAULT_SCRATCH_MEMORY_PER_SM as u32)
            as usize,
        SCRATCH_MEMORY_ALIGNMENT,
    )?;

    init_compute_engine_clean_state(
        &mut command_stream,
        BINDLESS_TEXTURE_CBUFF_INDEX,
        program_region.gpu_address(),
        &scratch_memory,
        gpu_characteristics.sm_arch_spa_version,
    )?;

    let src_res_buffer = GpuBox::new([0xCAFEu64; 0x2]);
    let copy_res_buffer = GpuBox::new([0x0u64; 0x2]);

    memcpy_1d(
        &mut command_stream,
        copy_res_buffer.gpu_address(),
        src_res_buffer.gpu_address(),
        src_res_buffer.user_size() as u32,
    )?;

    memcpy_inline_host_to_device(&mut command_stream, copy_res_buffer.gpu_address(), &[42])?;

    // Send the commands to the GPU.
    command_stream.flush()?;

    // Wait for the operations to be complete on the GPU side.
    command_stream.wait_idle();

    println!("copy_res_buffer: {:?}", &copy_res_buffer[..]);

    Ok(())
}
