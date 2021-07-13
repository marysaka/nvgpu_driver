use crate::utils::{Command, CommandStream, CommandSubmissionMode, SubChannelId};
use nvgpu::{GpuVirtualAddress, NvGpuResult};

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum DataTransferType {
    None,
    Pipelined,
    NonPipelined,
    Unknown(u32),
}

impl From<DataTransferType> for u32 {
    fn from(mode: DataTransferType) -> u32 {
        match mode {
            DataTransferType::None => 0,
            DataTransferType::Pipelined => 1,
            DataTransferType::NonPipelined => 2,
            DataTransferType::Unknown(val) => val,
        }
    }
}

impl From<u32> for DataTransferType {
    fn from(mode: u32) -> DataTransferType {
        match mode {
            0 => DataTransferType::None,
            1 => DataTransferType::Pipelined,
            2 => DataTransferType::NonPipelined,
            val => DataTransferType::Unknown(val),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum SemaphoreType {
    None,
    ReleaseOneWord,
    ReleaseFourWord,
    Unknown(u32),
}

impl From<SemaphoreType> for u32 {
    fn from(mode: SemaphoreType) -> u32 {
        match mode {
            SemaphoreType::None => 0,
            SemaphoreType::ReleaseOneWord => 1,
            SemaphoreType::ReleaseFourWord => 2,
            SemaphoreType::Unknown(val) => val,
        }
    }
}

impl From<u32> for SemaphoreType {
    fn from(mode: u32) -> SemaphoreType {
        match mode {
            0 => SemaphoreType::None,
            1 => SemaphoreType::ReleaseOneWord,
            2 => SemaphoreType::ReleaseFourWord,
            val => SemaphoreType::Unknown(val),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum InterruptType {
    None,
    Blocking,
    NonBlocking,
    Unknown(u32),
}

impl From<InterruptType> for u32 {
    fn from(mode: InterruptType) -> u32 {
        match mode {
            InterruptType::None => 0,
            InterruptType::Blocking => 1,
            InterruptType::NonBlocking => 2,
            InterruptType::Unknown(val) => val,
        }
    }
}

impl From<u32> for InterruptType {
    fn from(mode: u32) -> InterruptType {
        match mode {
            0 => InterruptType::None,
            1 => InterruptType::Blocking,
            2 => InterruptType::NonBlocking,
            val => InterruptType::Unknown(val),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum MemoryLayout {
    BlockLinear,
    Pitch,
    Unknown(u32),
}

impl From<MemoryLayout> for u32 {
    fn from(mode: MemoryLayout) -> u32 {
        match mode {
            MemoryLayout::BlockLinear => 0,
            MemoryLayout::Pitch => 1,
            MemoryLayout::Unknown(val) => val,
        }
    }
}

impl From<u32> for MemoryLayout {
    fn from(mode: u32) -> MemoryLayout {
        match mode {
            0 => MemoryLayout::BlockLinear,
            1 => MemoryLayout::Pitch,
            val => MemoryLayout::Unknown(val),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum MemoryType {
    Virtual,
    Physical,
    Unknown(u32),
}

impl From<MemoryType> for u32 {
    fn from(mode: MemoryType) -> u32 {
        match mode {
            MemoryType::Virtual => 0,
            MemoryType::Physical => 1,
            MemoryType::Unknown(val) => val,
        }
    }
}

impl From<u32> for MemoryType {
    fn from(mode: u32) -> MemoryType {
        match mode {
            0 => MemoryType::Virtual,
            1 => MemoryType::Physical,
            val => MemoryType::Unknown(val),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum SemaphoreReduction {
    IMin,
    IMax,
    IXor,
    IAnd,
    IOr,
    IAdd,
    Increment,
    Decrement,
    FAdd,
    Unknown(u32),
}

impl From<SemaphoreReduction> for u32 {
    fn from(mode: SemaphoreReduction) -> u32 {
        match mode {
            SemaphoreReduction::IMin => 0,
            SemaphoreReduction::IMax => 1,
            SemaphoreReduction::IXor => 2,
            SemaphoreReduction::IAnd => 3,
            SemaphoreReduction::IOr => 4,
            SemaphoreReduction::IAdd => 5,
            SemaphoreReduction::Increment => 6,
            SemaphoreReduction::Decrement => 7,
            // There is probably more here TODO poke this
            SemaphoreReduction::FAdd => 0xA,
            SemaphoreReduction::Unknown(val) => val,
        }
    }
}

impl From<u32> for SemaphoreReduction {
    fn from(mode: u32) -> SemaphoreReduction {
        match mode {
            0 => SemaphoreReduction::IMin,
            1 => SemaphoreReduction::IMax,
            2 => SemaphoreReduction::IXor,
            3 => SemaphoreReduction::IAnd,
            4 => SemaphoreReduction::IOr,
            5 => SemaphoreReduction::IAdd,
            6 => SemaphoreReduction::Increment,
            7 => SemaphoreReduction::Decrement,
            // There is probably more here TODO poke this
            0xA => SemaphoreReduction::FAdd,
            val => SemaphoreReduction::Unknown(val),
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum BypassL2 {
    UsePteSetting,
    ForceVolatile,
    Unknown(u32),
}

impl From<BypassL2> for u32 {
    fn from(mode: BypassL2) -> u32 {
        match mode {
            BypassL2::UsePteSetting => 0,
            BypassL2::ForceVolatile => 1,
            BypassL2::Unknown(val) => val,
        }
    }
}

impl From<u32> for BypassL2 {
    fn from(mode: u32) -> BypassL2 {
        match mode {
            0 => BypassL2::UsePteSetting,
            1 => BypassL2::ForceVolatile,
            val => BypassL2::Unknown(val),
        }
    }
}

bitfield! {
    pub struct LaunchDma(u32);
    impl Debug;

    #[inline]
    pub from into DataTransferType, data_transfer, set_data_transfer: 1, 0;

    #[inline]
    pub flush_enable, set_flush_enable: 2;

    #[inline]
    pub from into SemaphoreType, semaphore_type, set_semaphore_type: 4, 3;

    #[inline]
    pub from into InterruptType, interrupt_type, set_interrupt_type: 6, 5;

    #[inline]
    pub from into MemoryLayout, src_memory_layout, set_src_memory_layout: 7, 7;

    #[inline]
    pub from into MemoryLayout, dst_memory_layout, set_dst_memory_layout: 8, 8;

    #[inline]
    pub multi_line_enable, set_multi_line_enable: 9;

    #[inline]
    pub remap_emable, set_remap_emable: 10;

    // ???
    #[inline]
    pub rmw_disable, set_rmw_disable: 11;

    #[inline]
    pub from into MemoryType, src_type, set_src_type: 12, 12;

    #[inline]
    pub from into MemoryType, dst_type, set_dst_type: 13, 13;

    #[inline]
    pub from into SemaphoreReduction, semaphore_reduction, set_semaphore_reduction: 17, 14;

    #[inline]
    // TODO: enum this
    pub reduction_signed, set_reduction_signed: 18;

    #[inline]
    pub reduction_enable, set_reduction_enable: 19;

    #[inline]
    pub from into BypassL2, bypass_l2, set_bypass_l2: 20, 20;
}

impl LaunchDma {
    pub fn new() -> LaunchDma {
        LaunchDma(0)
    }
}

pub fn memcpy_1d(
    command_stream: &mut CommandStream,
    dst: GpuVirtualAddress,
    src: GpuVirtualAddress,
    size: u32,
) -> NvGpuResult<()> {
    // Setup lines to 1
    command_stream.push(Command::new_inline(
        0x107,
        SubChannelId::DirectMemoryAccess,
        1,
    ))?;

    let mut setup_dst = Command::new(
        0x1C5,
        SubChannelId::DirectMemoryAccess,
        CommandSubmissionMode::Increasing,
    );

    // Width = size
    setup_dst.push_argument(size);
    // Height = 1
    setup_dst.push_argument(1);
    // Depth = 0
    setup_dst.push_argument(0);

    command_stream.push(setup_dst)?;

    let mut setup_src = Command::new(
        0x1CC,
        SubChannelId::DirectMemoryAccess,
        CommandSubmissionMode::Increasing,
    );

    // Width = size
    setup_src.push_argument(size);
    // Height = 1
    setup_src.push_argument(1);
    // Depth = 0
    setup_src.push_argument(0);

    command_stream.push(setup_src)?;

    // Setup input and output address
    let mut setup_io = Command::new(
        0x100,
        SubChannelId::DirectMemoryAccess,
        CommandSubmissionMode::Increasing,
    );

    setup_io.push_address(src);
    setup_io.push_address(dst);

    command_stream.push(setup_io)?;

    let mut setup_line_len = Command::new(
        0x106,
        SubChannelId::DirectMemoryAccess,
        CommandSubmissionMode::Increasing,
    );

    // LineLengthIn = size
    setup_line_len.push_argument(size);
    command_stream.push(setup_line_len)?;

    let mut launch_dma_command = Command::new(
        0xC0,
        SubChannelId::DirectMemoryAccess,
        CommandSubmissionMode::Increasing,
    );

    let mut launch_dma = LaunchDma::new();

    launch_dma.set_data_transfer(DataTransferType::NonPipelined);
    launch_dma.set_flush_enable(true);
    launch_dma.set_src_memory_layout(MemoryLayout::Pitch);
    launch_dma.set_dst_memory_layout(MemoryLayout::Pitch);
    launch_dma.set_src_type(MemoryType::Virtual);
    launch_dma.set_dst_type(MemoryType::Virtual);

    launch_dma_command.push_argument(launch_dma.0);

    command_stream.push(launch_dma_command)?;

    Ok(())
}
