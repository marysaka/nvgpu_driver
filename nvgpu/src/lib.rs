//! Userland interface for nvgpu (Tegra graphics driver).
#[macro_use]
extern crate nix;

#[macro_use]
extern crate bitfield;

use nix::errno::Errno;
use nix::poll::{PollFd, PollFlags};
use nvhost::*;
use nvmap::*;

use std::fs::File;
use std::fs::OpenOptions;
use std::os::unix::io::AsRawFd;
use std::os::unix::io::FromRawFd;
use std::os::unix::io::RawFd;

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
#[allow(non_camel_case_types)]
pub enum ClassId {
    MAXWELL_B_3D,
    MAXWELL_B_COMPUTE,
    INLINE_TO_MEMORY,
    MAXWELL_A_2D,
    MAXWELL_B_DMA
}

impl From<ClassId> for u32 {
    fn from(class_id: ClassId) -> u32 {
        match class_id {
            ClassId::MAXWELL_B_3D => 0xB197,
            ClassId::MAXWELL_B_COMPUTE => 0xB1C0,
            ClassId::INLINE_TO_MEMORY => 0xA140,
            ClassId::MAXWELL_A_2D => 0x902D,
            ClassId::MAXWELL_B_DMA => 0xB0B5,
        }
    }
}

/// The result of NvGpu operations.
pub type NvGpuResult<T> = std::result::Result<T, Errno>;

/// Represent a virtual address in the GPU address space.
pub type GpuVirtualAddress = u64;

/// Represent an nvgpu address space instance.
pub struct AddressSpace {
    /// The inner file descriptor of this instance.
    file: File,
}

pub type GpFifoRawOffset = u64;

bitfield! {
  pub struct GpFifoEntry(u32);
  impl Debug;
  #[inline]
  pub method, set_method: 12, 0;

  #[inline]
  pub sub_channel, set_sub_channel: 15, 13;

  #[inline]
  pub argument_count, set_argument_count: 26, 16;

  #[inline]
  pub inline_arguments, set_inline_arguments: 26, 16;

  #[inline]
  pub unknown_28, set_unknown_28: 28;

  #[inline]
  pub submission_mode, set_submission_mode: 31, 29;
}

pub const GPFIFO_QUEUE_SIZE: usize = 0x800;

pub type GpFifoRawQueue = [GpFifoRawOffset; GPFIFO_QUEUE_SIZE];

pub struct GpFifoQueue<'a> {
    channel: &'a Channel,
    queue: GpFifoRawQueue,
    waiting_fence: Option<RawFence>,
    position: usize,
}

impl<'a> Drop for GpFifoQueue<'a> {
    fn drop(&mut self) {
        let _ = self.wait_idle();
    }
}

impl<'a> GpFifoQueue<'a> {
    pub fn new(channel: &'a Channel) -> Self {
        GpFifoQueue {
            channel,
            queue: [0; GPFIFO_QUEUE_SIZE],
            waiting_fence: None,
            position: 0,
        }
    }

    pub fn append(&mut self, gpu_address: GpuVirtualAddress, command_count: u64, _flags: u32) {
        if self.position >= GPFIFO_QUEUE_SIZE {
            panic!("No more space availaible in GpFifoCommandBuilder");
        }

        // TODO: use flags
        self.queue[self.position] = gpu_address | (command_count << 42);
        self.position += 1;
    }

    pub fn submit(&mut self) -> NvGpuResult<()> {
        let waiting_fence = self.waiting_fence.take();

        // 1 << 3 => fds
        let mut flags = 1 << 1 | 1 << 3;

        // We have something to wait on from past request.
        if waiting_fence.is_some() {
            flags |= 1;
        }

        self.waiting_fence =
            self.channel
                .submit_gpfifo(&self.queue[..self.position], waiting_fence, flags)?;

        self.position = 0;

        Ok(())
    }

    pub fn wait_idle(&mut self) -> nix::Result<()> {
        if let Some(fence) = self.waiting_fence.take() {
            let fd = fence.id as RawFd;

            let mut poll_fds = [PollFd::new(fd, PollFlags::POLLOUT | PollFlags::POLLIN)];

            nix::poll::poll(&mut poll_fds, -1)?;
        }

        Ok(())
    }
}

/// Represent an nvgpu channel.
pub struct Channel {
    /// The actual nvhost channel.
    inner: NvHostChannel,
}

pub const KIND_DEFAULT: i32 = -1;

#[allow(dead_code)]
mod ioctl {
    use super::GpFifoRawOffset;
    use super::GpuVirtualAddress;
    use super::RawFence;
    use std::os::unix::io::RawFd;

    /// NvGpuAs ioctl magic.
    const NVGPU_AS_IOCTL_MAGIC: u8 = b'A';

    /// NvHost/NvGpu ioctl magic.
    const NVGPU_IOCTL_MAGIC: u8 = b'H';

    /// NvGpu GPU ioctl magic.
    const NVGPU_GPU_IOCTL_MAGIC: u8 = b'G';

    /// NvGPU TSG ioctl magic.
    const NVGPU_TSG_IOCTL_MAGIC: u8 = b'T';

    /// Represent the structure of ``NVGPU_GPU_IOCTL_ALLOC_AS``.
    #[repr(C)]
    pub struct CtrlAllocAddressSpace {
        /// Input.
        pub big_page_size: u32,

        /// Output.
        pub as_fd: RawFd,

        /// Input.
        pub flags: u32,

        /// ???. must me zero.
        pub reserved: u32,
    }

    /// Represent the structure of ``NVGPU_GPU_IOCTL_OPEN_CHANNEL``.
    #[repr(C)]
    pub union CtrlOpenChannel {
        /// Input. -1 = default
        pub runlist_id: i32,

        /// Output.
        pub channel_fd: RawFd,
    }

    /// Represent the structure of ``NVGPU_GPU_IOCTL_OPEN_TSG``.
    #[repr(C)]
    pub struct CtrlOpenTSG {
        /// Output.
        pub tsg_fd: RawFd,

        /// reserved, must be 0.
        pub reserved: u32,
    }

    ioctl_readwrite!(
        ioc_ctrl_allocate_address_space,
        NVGPU_GPU_IOCTL_MAGIC,
        8,
        CtrlAllocAddressSpace
    );
    ioctl_readwrite!(ioc_ctrl_open_tsg, NVGPU_GPU_IOCTL_MAGIC, 9, CtrlOpenTSG);
    ioctl_readwrite!(
        ioc_ctrl_open_channel,
        NVGPU_GPU_IOCTL_MAGIC,
        11,
        CtrlOpenChannel
    );

    /// Represent the structure of ``NVGPU_AS_IOCTL_BIND_CHANNEL``.
    #[repr(C)]
    pub struct BindChannelArgument {
        pub channel_fd: RawFd,
    }

    /// Represent the structure of ``NVGPU_AS_IOCTL_UNMAP_BUFFER``
    #[repr(C)]
    pub struct UnmapBufferArguments {
        /// Input.
        pub offset: GpuVirtualAddress,
    }

    /// Represent the structure of ``NVGPU_AS_IOCTL_MAP_BUFFER_EX``.
    #[repr(C)]
    pub struct MapBufferExArguments {
        /// Input/Output.
        pub flags: u32,

        /// Input.
        pub compr_kind: i16,

        /// Input.
        pub incompr_kind: i16,

        /// Input.
        pub dmabuf_fd: RawFd,

        /// Input/Output.
        pub page_size: u32,

        /// Input/Output.
        pub buffer_offset: u64,

        /// Input/Output.
        pub mapping_size: u64,

        /// The virtual address to which the buffer is mapped to.
        /// Input/Output.
        pub offset: GpuVirtualAddress,
    }

    ioctl_readwrite!(
        ioc_as_bind_channel,
        NVGPU_AS_IOCTL_MAGIC,
        1,
        BindChannelArgument
    );
    ioctl_readwrite!(
        ioc_as_unmap_buffer,
        NVGPU_AS_IOCTL_MAGIC,
        5,
        UnmapBufferArguments
    );
    ioctl_readwrite!(
        ioc_as_map_buffer_ex,
        NVGPU_AS_IOCTL_MAGIC,
        7,
        MapBufferExArguments
    );

    /// Represent the structure of ``NVGPU_IOCTL_CHANNEL_ALLOC_GPFIFO``.
    #[repr(C)]
    pub struct ChannelAllocGpFifoArguments {
        pub num_entries: u32,
        pub flags: u32,
    }

    /// Represent the structure of ``NVGPU_IOCTL_CHANNEL_SUBMIT_GPFIFO``.
    #[repr(C)]
    pub struct ChannelSubmitGpFifoArguments {
        pub gpfifo: *const GpFifoRawOffset,
        pub num_entries: u32,
        pub flags: u32,
        pub fence: RawFence,
    }

    /// Represnet the structure of ``NVGPU_IOCTL_CHANNEL_ALLOC_OBJ_CTX``.
    #[repr(C)]
    pub struct ChannelAllocObjectContext {
        pub class_num: u32,
        pub flags: u32,
        pub obj_id: u64,
    }

    ioctl_write_ptr!(
        ioc_channel_alloc_gpfifo,
        NVGPU_IOCTL_MAGIC,
        100,
        ChannelAllocGpFifoArguments
    );
    ioctl_readwrite!(
        ioc_channel_submit_gpfifo,
        NVGPU_IOCTL_MAGIC,
        107,
        ChannelSubmitGpFifoArguments
    );
    ioctl_readwrite!(
        ioc_channel_alloc_object_context,
        NVGPU_IOCTL_MAGIC,
        108,
        ChannelAllocObjectContext
    );
    ioctl_none!(ioc_channel_enable, NVGPU_IOCTL_MAGIC, 113);
    ioctl_none!(ioc_channel_disable, NVGPU_IOCTL_MAGIC, 114);

    ioctl_write_ptr!(ioc_tsg_bind_channel, NVGPU_TSG_IOCTL_MAGIC, 1, RawFd);
    ioctl_write_ptr!(ioc_tsg_unbind_channel, NVGPU_TSG_IOCTL_MAGIC, 2, RawFd);
}

use ioctl::*;

/// Represent an instance of `/dev/nvhost-ctrl-gpu`.
pub struct NvHostGpuCtrl {
    /// The inner file descriptor of this instance.
    file: File,
}

impl NvHostGpuCtrl {
    /// Create a new instance of NvHostGpuCtrl by opening `/dev/nvhost-ctrl-gpu`.
    pub fn new() -> std::io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/nvhost-ctrl-gpu")?;
        Ok(NvHostGpuCtrl { file })
    }

    /// Create a new instance of NvHostGpuCtrl from a file descriptor.
    pub fn new_from_raw_fd(raw_fd: RawFd) -> Self {
        NvHostGpuCtrl {
            file: unsafe { File::from_raw_fd(raw_fd) },
        }
    }

    pub fn allocate_address_space(
        &self,
        big_page_size: u32,
        flags: u32,
    ) -> NvGpuResult<AddressSpace> {
        let mut param = CtrlAllocAddressSpace {
            big_page_size,
            as_fd: 0,
            flags,
            reserved: 0,
        };

        let res = unsafe { ioc_ctrl_allocate_address_space(self.file.as_raw_fd(), &mut param) };
        if res.is_err() {
            Err(Errno::UnknownErrno)
        } else {
            let errno = res.unwrap();
            if errno == 0 {
                Ok(AddressSpace::new_from_raw_fd(param.as_fd))
            } else {
                Err(Errno::from_i32(errno))
            }
        }
    }

    pub fn open_tsg(&self) -> NvGpuResult<TSGChannel> {
        let mut param = CtrlOpenTSG {
            tsg_fd: 0,
            reserved: 0,
        };

        let res = unsafe { ioc_ctrl_open_tsg(self.file.as_raw_fd(), &mut param) };
        if res.is_err() {
            Err(Errno::UnknownErrno)
        } else {
            let errno = res.unwrap();
            if errno == 0 {
                Ok(TSGChannel::new_from_raw_fd(param.tsg_fd))
            } else {
                Err(Errno::from_i32(errno))
            }
        }
    }

    pub fn open_channel(
        &self,
        runlist_id: i32,
        nvmap_instance: &NvMap,
        nvgpu_as: &AddressSpace,
        tsg: Option<&TSGChannel>,
    ) -> NvGpuResult<Channel> {
        let mut param = CtrlOpenChannel { runlist_id };

        let res = unsafe { ioc_ctrl_open_channel(self.file.as_raw_fd(), &mut param) };
        if res.is_err() {
            Err(Errno::UnknownErrno)
        } else {
            let errno = res.unwrap();
            if errno == 0 {
                Channel::new_from_raw_fd(unsafe { param.channel_fd }, nvmap_instance, nvgpu_as, tsg)
            } else {
                Err(Errno::from_i32(errno))
            }
        }
    }

    /// Get the file descriptor used.
    pub fn as_raw_fd(&self) -> RawFd {
        self.file.as_raw_fd()
    }
}

/// Represent an instance of `/dev/nvhost-tsg-gpu`.
pub struct TSGChannel {
    /// The inner file descriptor of this instance.
    file: File,
}

impl TSGChannel {
    /// Create a new instance of TSGChannel by opening `/dev/nvhost-tsg-gpu`.
    pub fn new() -> std::io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/nvhost-tsg-gpu")?;
        Ok(TSGChannel { file })
    }

    /// Create a new instance of TSGChannel from a file descriptor.
    pub fn new_from_raw_fd(raw_fd: RawFd) -> Self {
        TSGChannel {
            file: unsafe { File::from_raw_fd(raw_fd) },
        }
    }

    /// Get the file descriptor used.
    pub fn as_raw_fd(&self) -> RawFd {
        self.file.as_raw_fd()
    }

    pub fn bind_channel(&self, channel: &Channel) -> NvGpuResult<()> {
        let channel_fd = channel.as_raw_fd();
        let res = unsafe { ioc_tsg_bind_channel(self.file.as_raw_fd(), &channel_fd) };
        //let errno = unsafe { libc::ioctl(self.file.as_raw_fd(), 0x40045401, &arg as *const i64)  };

        if res.is_err() {
            Err(Errno::UnknownErrno)
        } else {
            let errno = res.unwrap();
            if errno == 0 {
                Ok(())
            } else {
                Err(Errno::from_i32(errno))
            }
        }
    }

    pub fn unbind_channel(&self, channel: &Channel) -> NvGpuResult<()> {
        let channel_fd = channel.as_raw_fd();
        let res = unsafe { ioc_tsg_unbind_channel(self.file.as_raw_fd(), &channel_fd) };
        //let errno = unsafe { libc::ioctl(self.file.as_raw_fd(), 0x40045401, &arg as *const i64)  };

        if res.is_err() {
            Err(Errno::UnknownErrno)
        } else {
            let errno = res.unwrap();
            if errno == 0 {
                Ok(())
            } else {
                Err(Errno::from_i32(errno))
            }
        }
    }
}

impl AddressSpace {
    /// Create a new instance of NvMap by opening `/dev/nvhost-as-gpu`.
    pub fn new() -> std::io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/nvhost-as-gpu")?;
        Ok(AddressSpace { file })
    }

    /// Create a new instance of NvMap from a file descriptor.
    pub fn new_from_raw_fd(raw_fd: RawFd) -> Self {
        AddressSpace {
            file: unsafe { File::from_raw_fd(raw_fd) },
        }
    }

    /// Get the file descriptor used.
    pub fn as_raw_fd(&self) -> RawFd {
        self.file.as_raw_fd()
    }

    pub fn bind_channel(&self, channel: &Channel) -> NvGpuResult<()> {
        let channel_fd = channel.as_raw_fd();
        let mut param = BindChannelArgument { channel_fd };

        let res = unsafe { ioc_as_bind_channel(self.file.as_raw_fd(), &mut param) };
        if res.is_err() {
            Err(Errno::UnknownErrno)
        } else {
            let errno = res.unwrap();
            if errno == 0 {
                Ok(())
            } else {
                Err(Errno::from_i32(errno))
            }
        }
    }

    pub fn map_buffer(
        &self,
        handle: &Handle,
        flags: u32,
        page_size: u32,
        fixed_address: GpuVirtualAddress,
    ) -> NvGpuResult<GpuVirtualAddress> {
        self.map_buffer_external(handle.fd, flags, 0, 0, page_size, 0, 0, fixed_address)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn map_buffer_external(
        &self,
        dmabuf_fd: RawFd,
        flags: u32,
        compr_kind: i16,
        incompr_kind: i16,
        page_size: u32,
        buffer_offset: u64,
        mapping_size: u64,
        fixed_address: GpuVirtualAddress,
    ) -> NvGpuResult<GpuVirtualAddress> {
        let mut param = MapBufferExArguments {
            flags: flags | (1 << 8),
            compr_kind,
            incompr_kind,
            dmabuf_fd,
            page_size,
            buffer_offset,
            mapping_size,
            offset: fixed_address,
        };

        let res = unsafe { ioc_as_map_buffer_ex(self.file.as_raw_fd(), &mut param) };
        if res.is_err() {
            Err(Errno::UnknownErrno)
        } else {
            let errno = res.unwrap();
            if errno == 0 {
                Ok(param.offset)
            } else {
                Err(Errno::from_i32(errno))
            }
        }
    }

    pub fn unmap_buffer(&self, address: GpuVirtualAddress) -> NvGpuResult<()> {
        let mut param = UnmapBufferArguments { offset: address };

        let res = unsafe { ioc_as_unmap_buffer(self.file.as_raw_fd(), &mut param) };
        if res.is_err() {
            Err(Errno::UnknownErrno)
        } else {
            let errno = res.unwrap();
            if errno == 0 {
                Ok(())
            } else {
                Err(Errno::from_i32(errno))
            }
        }
    }
}

impl Channel {
    /// Create a new instance of Channel by opening `/dev/nvhost-gpu`.
    pub fn new(nvmap_instance: &NvMap, nvgpu_as: &AddressSpace) -> NvGpuResult<Self> {
        Self::new_from_path("/dev/nvhost-gpu", nvmap_instance, nvgpu_as)
    }

    pub fn new_from_path(
        path: &str,
        nvmap_instance: &NvMap,
        nvgpu_as: &AddressSpace,
    ) -> NvGpuResult<Self> {
        let nvhost_channel =
            NvHostChannel::new(path, nvmap_instance).expect("Cannot open GPU channel");
        let mut channel = Channel {
            inner: nvhost_channel,
        };
        nvgpu_as.bind_channel(&channel)?;
        channel.allocate_gpfifo(GPFIFO_QUEUE_SIZE, 0)?;
        channel.allocate_object_context(ClassId::MAXWELL_B_3D, 0x0)?;
        Ok(channel)
    }

    /// Create a new instance of NvMap from a file descriptor.
    pub fn new_from_raw_fd(
        raw_fd: RawFd,
        nvmap_instance: &NvMap,
        nvgpu_as: &AddressSpace,
        tsg: Option<&TSGChannel>,
    ) -> NvGpuResult<Self> {
        let nvhost_channel = NvHostChannel::new_from_raw_fd(raw_fd, nvmap_instance)?;
        let mut channel = Channel {
            inner: nvhost_channel,
        };

        if let Some(tsg) = tsg {
            tsg.bind_channel(&channel)?;
        } else {
            channel.set_priority(ChannelPriority::Medium)?;
        }

        nvgpu_as.bind_channel(&channel)?;
        channel.allocate_gpfifo(GPFIFO_QUEUE_SIZE, 0)?;
        channel.allocate_object_context(ClassId::MAXWELL_B_3D, 0x0)?;
        Ok(channel)
    }

    pub fn set_priority(&self, priority: ChannelPriority) -> NvGpuResult<()> {
        self.inner.set_priority(priority)
    }

    pub fn allocate_gpfifo(&mut self, gpfifo_queue_size: usize, flags: u32) -> NvGpuResult<()> {
        let param = ChannelAllocGpFifoArguments {
            num_entries: gpfifo_queue_size as u32,
            flags,
        };

        let res = unsafe { ioc_channel_alloc_gpfifo(self.inner.as_raw_fd(), &param) };
        if res.is_err() {
            Err(Errno::UnknownErrno)
        } else {
            let errno = res.unwrap();
            if errno == 0 {
                Ok(())
            } else {
                Err(Errno::from_i32(errno))
            }
        }
    }

    pub fn submit_gpfifo(
        &self,
        entries: &[GpFifoRawOffset],
        input_fence: Option<RawFence>,
        flags: u32,
    ) -> NvGpuResult<Option<RawFence>> {
        let input_fence = input_fence.unwrap_or_else(|| RawFence {
            id: -1,
            value: 0xFFFF_FFFF,
        });

        let mut param = ChannelSubmitGpFifoArguments {
            gpfifo: entries.as_ptr(),
            num_entries: entries.len() as u32,
            flags,
            fence: input_fence,
        };

        let res = unsafe { ioc_channel_submit_gpfifo(self.inner.as_raw_fd(), &mut param) };
        if res.is_err() {
            Err(Errno::UnknownErrno)
        } else {
            let errno = res.unwrap();
            if errno == 0 {
                let output_fence = if flags & (1 << 1) != 0 {
                    Some(param.fence)
                } else {
                    None
                };
                Ok(output_fence)
            } else {
                Err(Errno::from_i32(errno))
            }
        }
    }

    pub fn allocate_object_context(&mut self, class_num: ClassId, flags: u32) -> NvGpuResult<u64> {
        let mut param = ChannelAllocObjectContext {
            class_num: u32::from(class_num),
            flags,
            obj_id: 0,
        };

        let res = unsafe { ioc_channel_alloc_object_context(self.inner.as_raw_fd(), &mut param) };
        if res.is_err() {
            Err(Errno::UnknownErrno)
        } else {
            let errno = res.unwrap();
            if errno == 0 {
                Ok(param.obj_id)
            } else {
                Err(Errno::from_i32(errno))
            }
        }
    }

    pub fn enable(&self) -> NvGpuResult<()> {
        let res = unsafe { ioc_channel_enable(self.inner.as_raw_fd()) };
        if res.is_err() {
            Err(Errno::UnknownErrno)
        } else {
            let errno = res.unwrap();
            if errno == 0 {
                Ok(())
            } else {
                Err(Errno::from_i32(errno))
            }
        }
    }

    pub fn disable(&self) -> NvGpuResult<()> {
        let res = unsafe { ioc_channel_disable(self.inner.as_raw_fd()) };
        if res.is_err() {
            Err(Errno::UnknownErrno)
        } else {
            let errno = res.unwrap();
            if errno == 0 {
                Ok(())
            } else {
                Err(Errno::from_i32(errno))
            }
        }
    }

    /// Get the file descriptor used.
    pub fn as_raw_fd(&self) -> RawFd {
        self.inner.as_raw_fd()
    }
}
