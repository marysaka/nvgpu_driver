//! Userland interface for nvhost (Tegra graphics host driver).
#[macro_use]
extern crate nix;

use nix::errno::Errno;
use nvmap::NvMap;

use std::fs::File;
use std::fs::OpenOptions;
use std::os::unix::io::AsRawFd;
use std::os::unix::io::FromRawFd;
use std::os::unix::io::RawFd;

/// Represent a SyncPoint identifier.
pub type SyncPointId = i32;

/// Represent the raw representation of a fence
#[repr(C)]
#[derive(Debug)]
pub struct RawFence {
    pub id: SyncPointId,
    pub value: u32,
}

/// Represent an instance of `/dev/nvhost-ctrl`.
pub struct NvHostCtrl {
    /// The inner file descriptor of this instance.
    file: File,
}

/// Represent an instance of an nvhost channel
pub struct NvHostChannel {
    /// The inner file descriptor of this instance.
    file: File,
}

/// The result of NvHost operations.
pub type NvHostResult<T> = std::result::Result<T, Errno>;

#[repr(C)]
pub struct SyncFenceInfo {
    id: SyncPointId,
    threshhold: u32,
}

#[repr(C)]
pub struct Characteristics {
    flags: u64,
    num_mlocks: u32,
    num_syncpts: u32,
    syncpts_base: u32,
    syncpts_limit: u32,
    num_hw_pts: u32,
    padding: u32,
}

#[repr(packed)]
pub struct CommandBuffer {
    pub mem: u32,
    pub offset: u32,
    pub words: u32,
}

#[repr(C)]
pub struct CommandBufferExt {
    pub pre_fence: i32,
    reserved: u32,
}

#[repr(C)]
pub struct Relocation {
    pub cmdbuf_mem: u32,
    pub cmdbuf_offset: u32,
    pub target: u32,
    pub target_offset: u32,
}

#[repr(C)]
pub struct RelocationType {
    pub reloc_type: u32,
    padding: u32,
}

#[repr(packed)]
pub struct RelocationShift {
    pub shift: u32,
}

#[repr(C)]
pub struct WaitChk {
    pub mem: u32,
    pub offset: u32,
    pub syncpoint_id: SyncPointId,
    pub threshhold: u32,
}

#[repr(C)]
pub struct SyncPointIncrement {
    pub syncpoint_id: SyncPointId,
    pub syncpoint_incrs: u32,
}

/// Channel priority used in [NvHost::set_priority]
///
/// [NvHost::set_priority]: struct.NvHost.html#method.set_priority
pub enum ChannelPriority {
    Low,
    Medium,
    High,
}

impl From<ChannelPriority> for u32 {
    fn from(input: ChannelPriority) -> Self {
        match input {
            ChannelPriority::Low => 50,
            ChannelPriority::Medium => 100,
            ChannelPriority::High => 150,
        }
    }
}

/// NvHost IOCTLs
#[allow(dead_code)]
mod ioctl {
    use std::os::unix::io::RawFd;
    use super::Characteristics;
    use super::CommandBuffer;
    use super::CommandBufferExt;
    use super::RawFence;
    use super::Relocation;
    use super::RelocationShift;
    use super::RelocationType;
    use super::SyncFenceInfo;
    use super::SyncPointId;
    use super::SyncPointIncrement;
    use super::WaitChk;


    /// NvHost ioctl magic.
    const NVHOST_IOCTL_MAGIC: u8 = b'H';

    /// Represent the structure of ``NVHOST_IOCTL_CTRL_SYNCPT_INCR``.
    #[repr(C)]
    pub struct SyncPointDoIncrement {
        pub id: SyncPointId,
    }

    /// Represent the structure of ``NVHOST_IOCTL_CTRL_SYNCPT_WAIT``.
    #[repr(packed)]
    pub struct SyncPointWait {
        pub id: SyncPointId,
        pub threshhold: u32,
        pub timeout: i32,
    }

    /// Represent the structure of ``NVHOST_IOCTL_CTRL_MODULE_MUTEX``.
    #[repr(C)]
    pub struct ModuleMutex {
        pub id: SyncPointId,
        pub lock: u32,
    }

    /// Represent the structure of ``NVHOST_IOCTL_CTRL_SYNCPT_WAITEX``.
    #[repr(C)]
    pub struct SyncPointWaitEx {
        pub id: SyncPointId,
        pub threshhold: u32,
        pub timeout: i32,
        pub value: u32,
    }

    /// Represent the structure of ``NVHOST_IOCTL_CTRL_GET_VERSION``,
    /// ``NVHOST_IOCTL_CHANNEL_GET_SYNCPOINTS``, ``NVHOST_IOCTL_CHANNEL_GET_WAITBASES``,
    /// ``NVHOST_IOCTL_CHANNEL_GET_MODMUTEXES``, ``NVHOST_IOCTL_CHANNEL_NULL_KICKOFF``,
    /// and ``NVHOST_IOCTL_CHANNEL_GET_TIMEDOUT``.
    #[repr(C)]
    pub struct GetParamArguments {
        pub value: u32,
    }

    /// Represent the structure of ``NVHOST_IOCTL_CTRL_SYNCPT_WAITMEX``.
    #[repr(C)]
    pub struct SyncPointWaitMEx {
        pub id: SyncPointId,
        pub threshhold: u32,
        pub timeout: i32,
        pub value: u32,
        pub tv_sec: u32,
        pub tv_nsec: u32,
        reserved_1: u32,
        reserved_2: u32,
    }

    /// Represent the structure of ``NVHOST_IOCTL_CTRL_MODULE_REGRDWR`` and ``NVHOST_IOCTL_CHANNEL_MODULE_REGRDWR``.
    #[repr(C)]
    pub struct ModuleRegisterReadWrite {
        pub id: SyncPointId,
        pub num_offsets: u32,
        pub block_size: u32,
        pub write: u32,
        pub offsets: u64,
        pub values: u64,
    }

    /// Represent the structure of ``NVHOST_IOCTL_CTRL_SYNC_FENCE_CREATE``.
    #[repr(C)]
    pub struct SyncFenceCreate {
        pub num_pts: u32,
        pub fence_fd: i32,
        pub pts: *const SyncFenceInfo,
        pub name: *const u8,
    }

    /// Represent the structure of ``NVHOST_IOCTL_CTRL_SYNC_FENCE_SET_NAME``.
    #[repr(C)]
    pub struct SyncFenceSetName {
        pub name: *const u8,
        pub fence_fd: i32,
    }

    /// Represent the structure of ``NVHOST_IOCTL_CTRL_GET_CHARACTERISTICS``.
    #[repr(C)]
    pub struct GetCharacteristics {
        pub characteristics_size: u64,
        pub characteristics_address: *const Characteristics,
    }

    /// Represent the structure of ``NVHOST_IOCTL_CTRL_CHECK_MODULE_SUPPORT``.
    #[repr(C)]
    pub struct CheckModuleSupport {
        pub module_id: u32,
        pub value: u32,
    }

    // Ctrl IOCTLs
    ioctl_readwrite!(ioc_ctrl_syncpoint_read, NVHOST_IOCTL_MAGIC, 1, RawFence);
    ioctl_write_ptr!(
        ioc_ctrl_syncpoint_increment,
        NVHOST_IOCTL_MAGIC,
        2,
        SyncPointDoIncrement
    );
    ioctl_write_ptr!(
        ioc_ctrl_syncpoint_wait,
        NVHOST_IOCTL_MAGIC,
        3,
        SyncPointWait
    );
    ioctl_readwrite!(ioc_ctrl_module_mutex, NVHOST_IOCTL_MAGIC, 4, ModuleMutex);
    ioctl_write_ptr!(
        ioc_ctrl_syncpoint_waitex,
        NVHOST_IOCTL_MAGIC,
        6,
        SyncPointWaitEx
    );
    ioctl_read!(
        ioc_ctrl_get_version,
        NVHOST_IOCTL_MAGIC,
        7,
        GetParamArguments
    );
    ioctl_readwrite!(ioc_ctrl_syncpoint_read_max, NVHOST_IOCTL_MAGIC, 8, RawFence);
    ioctl_readwrite!(
        ioc_ctrl_syncpoint_waitmex,
        NVHOST_IOCTL_MAGIC,
        9,
        SyncPointWaitMEx
    );
    ioctl_readwrite!(
        ioc_ctrl_sync_fence_create,
        NVHOST_IOCTL_MAGIC,
        11,
        SyncFenceCreate
    );
    ioctl_readwrite!(
        ioc_ctrl_module_register_readwrite,
        NVHOST_IOCTL_MAGIC,
        12,
        ModuleRegisterReadWrite
    );
    ioctl_readwrite!(
        ioc_ctrl_sync_fence_set_name,
        NVHOST_IOCTL_MAGIC,
        13,
        SyncFenceSetName
    );
    ioctl_readwrite!(
        ioc_ctrl_get_characteristics,
        NVHOST_IOCTL_MAGIC,
        14,
        GetCharacteristics
    );
    ioctl_readwrite!(
        ioc_ctrl_check_module_support,
        NVHOST_IOCTL_MAGIC,
        15,
        CheckModuleSupport
    );

    /// Represent the structure of ``NVHOST_IOCTL_CHANNEL_SET_NVMAP_FD``.
    #[repr(packed)]
    pub struct SetNvMapFdArguments {
        pub fd: RawFd,
    }

    /// Represent the structure of ``NVHOST_IOCTL_CHANNEL_GET_CLK_RATE`` and ``NVHOST_IOCTL_CHANNEL_SET_CLK_RATE``.
    #[repr(C)]
    pub struct ClockRateArguments {
        pub rate: u32,
        pub module_id: u32,
    }

    /// Represent the structure of ``NVHOST_IOCTL_CHANNEL_SET_TIMEOUT``.
    #[repr(packed)]
    pub struct SetTimeoutArguments {
        pub timeout: u32,
    }

    /// Represent the structure of ``NVHOST_IOCTL_CHANNEL_SET_TIMESLICE``.
    #[repr(packed)]
    pub struct SetTimeSliceArguments {
        pub timeslice_us: u32,
        pub reserved: u32,
    }

    /// Represent the structure of ``NVHOST_IOCTL_CHANNEL_GET_SYNCPOINT`` and ``NVHOST_IOCTL_CHANNEL_GET_MODMUTEX``.
    #[repr(C)]
    pub struct GetParamValueArgument {
        /// The parameter to use (Input).
        pub param: u32,

        /// The resulting value (Output).
        pub value: u32,
    }

    /// Represent the structure of ``NVHOST_IOCTL_CHANNEL_SET_TIMEOUT_EX``.
    #[repr(C)]
    pub struct SetTimeoutExArguments {
        pub timeout: u32,
        pub flags: u32,
    }

    /// Represent the structure of ``NVHOST_IOCTL_CHANNEL_GET_CLIENT_MANAGED_SYNCPOINT``.
    #[repr(C)]
    pub struct GetClientManagedSyncPointArgument {
        pub name: *const u8,
        pub param: u32,
        pub value: u32,
    }

    /// Represent the structure of ``NVHOST_IOCTL_CHANNEL_SET_CTXSWITCH``.
    #[repr(C)]
    pub struct SetContextSwitchArguments {
        pub num_cmdbufs_save: u32,
        pub num_save_incrs: u32,
        pub save_incrs: u32,
        pub save_waitbases: u32,
        pub cmdbuf_save: u32,
        pub num_cmdbufs_restore: u32,
        pub num_restore_incrs: u32,
        pub restore_incrs: u32,
        pub restore_waitbases: u32,
        pub cmdbuf_restore: u32,
        pub num_relocs: u32,
        pub relocs: u32,
        pub reloc_shifts: u32,
        padding: u32,
    }

    /// Represent the structure of ``NVHOST_IOCTL_CHANNEL_SUBMIT``.
    #[repr(C)]
    pub struct SubmitArguments {
        pub submit_version: u32,
        pub num_syncpt_incrs: u32,
        pub num_cmdbufs: u32,
        pub num_relocs: u32,
        pub num_waitchks: u32,
        pub timeout: u32,
        pub flags: u32,

        /// Out
        pub fence: u32,
        pub syncpt_incrs: *const SyncPointIncrement,
        pub cmdbuf_exts: *const CommandBufferExt,

        pub checksum_methods: u32,
        pub checksum_falcon_methods: u32,

        pub reserved_for_future_use: u64,

        pub reloc_types: *const RelocationType,
        pub cmdbufs: *const CommandBuffer,
        pub relocs: *const Relocation,
        pub reloc_shifts: *const RelocationShift,
        pub waitchks: *const WaitChk,
        // ignored by the driver???
        pub waitbases: u64,
        pub class_ids: *const u32,
        /// fence fds
        pub fences: *const SyncPointId,
    }

    /// Represent the structure of ``NVHOST_IOCTL_CHANNEL_SET_SYNCPOINT_NAME``.
    #[repr(C)]
    pub struct SetSyncPointNameArguments {
        pub name: *const u8,
        pub syncpoint_id: SyncPointId,
        padding: u32,
    }

    /// Represnet the structure of ``NVHOST_IOCTL_CHANNEL_SET_ERROR_NOTIFIER``.
    #[repr(C)]
    pub struct SetErrorNotifier {
        pub offset: u64,
        pub size: u64,
        pub mem: u32,
        padding: u32,
    }

    /// Represent the structure of ``NVHOST_IOCTL_CHANNEL_OPEN``.
    #[repr(C)]
    pub struct ChannelOpen {
        channel_fd: i32,
    }

    // Channel IOCTLs
    ioctl_read!(
        ioc_channel_get_syncpoints,
        NVHOST_IOCTL_MAGIC,
        2,
        GetParamArguments
    );
    ioctl_read!(
        ioc_channel_get_waitbases,
        NVHOST_IOCTL_MAGIC,
        3,
        GetParamArguments
    );
    ioctl_read!(
        ioc_channel_get_modmutexes,
        NVHOST_IOCTL_MAGIC,
        4,
        GetParamArguments
    );
    ioctl_write_ptr!(
        ioc_channel_set_nvmap_fd,
        NVHOST_IOCTL_MAGIC,
        5,
        SetNvMapFdArguments
    );
    ioctl_read!(
        ioc_channel_null_kickoff,
        NVHOST_IOCTL_MAGIC,
        6,
        GetParamArguments
    );
    ioctl_readwrite!(
        ioc_channel_get_clock_rate,
        NVHOST_IOCTL_MAGIC,
        9,
        ClockRateArguments
    );
    ioctl_write_ptr!(
        ioc_channel_set_clock_rate,
        NVHOST_IOCTL_MAGIC,
        10,
        ClockRateArguments
    );
    ioctl_write_ptr!(
        ioc_channel_set_timeout,
        NVHOST_IOCTL_MAGIC,
        11,
        SetTimeoutArguments
    );
    ioctl_read!(
        ioc_channel_get_timeout,
        NVHOST_IOCTL_MAGIC,
        12,
        GetParamArguments
    );
    ioctl_readwrite!(
        ioc_channel_get_syncpoint,
        NVHOST_IOCTL_MAGIC,
        16,
        GetParamValueArgument
    );
    ioctl_readwrite!(
        ioc_channel_set_timeout_ex,
        NVHOST_IOCTL_MAGIC,
        18,
        SetTimeoutExArguments
    );
    ioctl_readwrite!(
        ioc_channel_get_client_managed_syncpoint,
        NVHOST_IOCTL_MAGIC,
        19,
        GetClientManagedSyncPointArgument
    );
    ioctl_readwrite!(
        ioc_channel_get_modmutex,
        NVHOST_IOCTL_MAGIC,
        23,
        GetParamValueArgument
    );
    ioctl_readwrite!(
        ioc_channel_set_context_switch,
        NVHOST_IOCTL_MAGIC,
        25,
        SetContextSwitchArguments
    );

    ioctl_readwrite!(ioc_channel_submit, NVHOST_IOCTL_MAGIC, 26, SubmitArguments);
    ioctl_readwrite!(
        ioc_channel_module_register_readwrite,
        NVHOST_IOCTL_MAGIC,
        27,
        ModuleRegisterReadWrite
    );
    ioctl_write_ptr!(
        ioc_channel_set_syncpoint_name,
        NVHOST_IOCTL_MAGIC,
        30,
        SetSyncPointNameArguments
    );

    ioctl_readwrite!(
        ioc_channel_set_error_notifier,
        NVHOST_IOCTL_MAGIC,
        111,
        SetErrorNotifier
    );
    ioctl_write_ptr!(ioc_channel_open, NVHOST_IOCTL_MAGIC, 112, ChannelOpen);

    ioctl_write_ptr!(
        ioc_channel_set_timeslice,
        NVHOST_IOCTL_MAGIC,
        121,
        SetTimeSliceArguments
    );
}

use ioctl::*;

impl NvHostCtrl {
    /// Create a new instance of NvHostCtrl by opening `/dev/nvhost-ctrl`.
    pub fn new() -> std::io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/nvhost-ctrl")?;
        Ok(NvHostCtrl { file })
    }

    /// Create a new instance of NvHostCtrl from a file descriptor.
    pub fn new_from_raw_fd(raw_fd: RawFd) -> Self {
        NvHostCtrl {
            file: unsafe { File::from_raw_fd(raw_fd) },
        }
    }

    /// Get the file descriptor used.
    pub fn as_raw_fd(&self) -> RawFd {
        self.file.as_raw_fd()
    }
}

impl NvHostChannel {
    /// Create a new instance of NvHostChannel by opening the given path and an nvmap instance.
    pub fn new(path: &str, nvmap_instance: &NvMap) -> NvHostResult<Self> {
        let file = OpenOptions::new().read(true).write(true).open(path);
        if file.is_err() {
            return Err(Errno::ENOENT);
        }
        let file = file.unwrap();
        let res = NvHostChannel { file };

        res.set_nvmap_fd(nvmap_instance.as_raw_fd())?;
        Ok(res)
    }

    /// Create a new instance of NvHostChannel from a file descriptor and an nvmap instance.
    pub fn new_from_raw_fd(raw_fd: RawFd, nvmap_instance: &NvMap) -> NvHostResult<Self> {
        let res = NvHostChannel {
            file: unsafe { File::from_raw_fd(raw_fd) },
        };
        res.set_nvmap_fd(nvmap_instance.as_raw_fd())?;

        Ok(res)
    }

    /// Assign the given nvmap file descriptor to this channel.
    pub fn set_nvmap_fd(&self, fd: RawFd) -> NvHostResult<()> {
        let param = SetNvMapFdArguments { fd };

        let res = unsafe { ioc_channel_set_nvmap_fd(self.file.as_raw_fd(), &param) };
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

    pub fn set_priority(&self, priority: ChannelPriority) -> NvHostResult<()> {
        let timeslice_us = match priority {
            ChannelPriority::High => 5200,
            ChannelPriority::Medium => 2600,
            ChannelPriority::Low => 1300,
        };

        self.set_timeslice(timeslice_us)
    }

    pub fn set_timeslice(&self, timeslice_us: u32) -> NvHostResult<()> {
        let param = SetTimeSliceArguments {
            timeslice_us,
            reserved: 0,
        };

        let res = unsafe { ioc_channel_set_timeslice(self.file.as_raw_fd(), &param) };
        if res.is_err() {
            // FIXME: this is unimplemented on R32.2
            if let Err(nix::Error::Sys(Errno::ENOTTY)) = res {
                return Ok(());
            }
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

    ///pub fn set_error_notifier(&self, )

    /// Get the file descriptor used.
    pub fn as_raw_fd(&self) -> RawFd {
        self.file.as_raw_fd()
    }
}
