use crate::utils::{Command, CommandStream, CommandSubmissionMode, SubChannelId};
use nvgpu::{GpuVirtualAddress, NvGpuResult};

pub fn memcpy_inline_host_to_device(
    command_stream: &mut CommandStream,
    dst: GpuVirtualAddress,
    data: &[u8],
) -> NvGpuResult<()> {
    // Setup dst and size.
    let mut setup_dst = Command::new(
        0x60,
        SubChannelId::Compute,
        CommandSubmissionMode::Increasing,
    );

    setup_dst.push_argument(data.len() as u32);
    setup_dst.push_argument(1);
    setup_dst.push_address(dst);

    command_stream.push(setup_dst)?;

    let mut launch_dma_command = Command::new(
        0x6C,
        SubChannelId::Compute,
        CommandSubmissionMode::Increasing,
    );

    // TODO: map to bitfield
    launch_dma_command.push_argument(0x11);

    command_stream.push(launch_dma_command)?;

    // Finally send inline data

    let mut inline_data = Command::new(
        0x6D,
        SubChannelId::Compute,
        CommandSubmissionMode::NonIncreasing,
    );
    inline_data.push_inlined_buffer(data);

    command_stream.push(inline_data)?;

    Ok(())
}
