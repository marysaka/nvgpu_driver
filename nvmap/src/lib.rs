//! Userland interface for nvmap (Memory manager for Tegra GPU).
#[macro_use]
extern crate nix;

use bitflags::bitflags;

use nix::errno::Errno;

use std::fs::File;
use std::fs::OpenOptions;
use std::os::unix::io::AsRawFd;
use std::os::unix::io::FromRawFd;
use std::os::unix::io::RawFd;

use nix::sys::mman::*;

/// This is the raw representation of a NvMap handle.
///
/// NOTE: this is the handle returned by the driver.
pub type RawHandle = u32;

/// High level representation of a NvMap handle.
#[derive(Debug)]
pub struct Handle {
    /// The size of the memory region behind the memory handle.
    pub size: u32,

    /// The memory handle.
    pub raw_handle: RawHandle,

    /// The file descriptor associated to this handle.
    pub fd: RawFd,

    /// The mapped address of the memory handle.
    mapped_address: Option<*mut u8>,
}

/// The result of NvMap operations.
pub type NvMapResult<T> = std::result::Result<T, Errno>;

/// Represent an NvMap instance.
pub struct NvMap {
    /// The inner file descriptor of this instance.
    file: File,
}

impl Handle {
    /// Get the size of the memory region behind the memory handle.
    pub fn size(&self) -> u32 {
        self.size
    }

    /// Get the mapped address of the memory handle.
    ///
    /// NOTE: the resource can be mapped using [NvMap::map]
    ///
    /// [NvMap::map]: struct.NvMap.html#method.map
    pub fn addr(&self) -> Option<*mut u8> {
        self.mapped_address
    }

    /// Creater a new Handle instance.
    ///
    /// NOTE: to allocate a new Handle please use [NvMap::create]
    ///
    /// [NvMap::create]: struct.NvMap.html#method.create
    pub fn from_raw(raw_handle: RawHandle, fd: RawFd, size: u32) -> Self {
        Handle {
            size,
            raw_handle,
            fd,
            mapped_address: None,
        }
    }
}

// TODO: structs for flags.

bitflags! {
    /// Heap mask used in [NvMap::allocate]
    ///
    /// [NvMap::allocate]: struct.NvMap.html#method.allocate
    pub struct HeapMask: u32 {
        /// I/O Virtual Memory Manager.
        const IOVMM = 1 << 30;

        /// IRAM Heap carveout.
        const CARVEOUT_IRAM = 1 << 29;

        /// VPR Heap carveout.
        const CARVEOUT_VPR = 1 << 28;

        /// Tegra Security Co-processor Heap carveout.
        const CARVEOUT_TSEC = 1 << 27;

        /// Video Memory Heap carveout.
        const CARVEOUT_VIDMEM = 1 << 26;

        /// IVM Heap carveout.
        const CARVEOUT_IVM = 1 << 1;

        /// Generic Heap carveout.
        const CARVEOUT_GENERIC = 1;
    }
}

bitflags! {
    /// Allocation flags used in [NvMap::allocate]
    ///
    /// TODO: Support tags to avoid warnings on the kernel side.
    ///
    /// [NvMap::allocate]: struct.NvMap.html#method.allocate
    pub struct AllocationFlags: u32 {
        /// Flag the allocated region as uncacheable.
        const HANDLE_UNCACHEABLE = 0b0;

        /// Flag the allocated region as write combine.
        const HANDLE_WRITE_COMBINE = 0b1;

        /// Flag the allocated region as inner cacheable.
        const HANDLE_INNER_CACHEABLE = 0b10;

        /// Flag the allocated region as inner/outer cacheable.
        const HANDLE_CACHEABLE = 0b11;
    }
}

/// Flush operation flag for ``NVMAP_IOC_CACHE``.
const CACHE_OPERATION_WRITE_BACK: i32 = 0;

/// Invalidate operation flag for ``NVMAP_IOC_CACHE``.
const CACHE_OPERATION_INVALIDATE: i32 = 1;

/// Flush & Invalidate operation flag for ``NVMAP_IOC_CACHE``.
const CACHE_OPERATION_WRITE_BACK_INVALIDATE: i32 = 2;

/// Internal module managing raw ioctls calls.
mod ioctl {
    /// The IOCTL magic of NvMap
    const NVMAP_IOC_MAGIC: u8 = b'N';

    use super::RawHandle;

    /// Structure for ``NVMAP_IOC_CREATE``.
    #[repr(C)]
    pub struct CreateHandle {
        /// The size wanted by the client. (Input)
        pub size: u32,

        /// The resulting memory handle. (Output)
        pub handle: RawHandle,
    }

    /// Structure for ``NVMAP_IOC_GET_FD``.
    #[repr(C)]
    pub struct HandleGetFd {
        /// The resulting file descriptor associated to the memory handle. (Output)
        pub fd: i32,

        /// The handle requiring its file descriptor. (Input)
        pub handle: RawHandle,
    }

    /// Structure for ``NVMAP_IOC_FROM_FD``.
    #[repr(C)]
    pub struct CreateHandleFromFd {
        /// The file descriptor to use for the backed memory. (Input)
        pub fd: i32,

        /// The resulting memory handle. (Output)
        pub handle: RawHandle,
    }

    /// Structure for ``NVMAP_IOC_CACHE``.
    #[repr(C)]
    pub struct HandleCacheMaintenance {
        /// The pointer to the memory region to do maintenance on. (Input)
        pub address: u64,

        /// The associated memory handle. (Input)
        pub handle: RawHandle,

        /// The length used for the maintenance. (Input)
        pub length: u32,

        /// The cache maintenance operation to apply. (Input)
        pub operation: i32,
    }

    /// Structure for ``NVMAP_IOC_ALLOC``.
    #[repr(C)]
    pub struct AllocateHandle {
        /// The memory handle that needs memory. (Input)
        pub handle: RawHandle,

        /// The heap to allocate from. (Input)
        pub heap_mask: u32,

        /// The flags of the memory region. (Input)
        pub flags: u32,

        /// The alignment needed. (Input)
        pub align: u32,
    }

    ioctl_readwrite!(ioc_create, NVMAP_IOC_MAGIC, 0, CreateHandle);
    ioctl_write_ptr!(ioc_allocate, NVMAP_IOC_MAGIC, 3, AllocateHandle);
    ioctl_write_ptr!(ioc_cache, NVMAP_IOC_MAGIC, 12, HandleCacheMaintenance);
    ioctl_readwrite!(ioc_get_fd, NVMAP_IOC_MAGIC, 15, HandleGetFd);
    ioctl_readwrite!(ioc_from_fd, NVMAP_IOC_MAGIC, 16, CreateHandleFromFd);
    ioctl_write_int_bad!(ioc_free, request_code_none!(NVMAP_IOC_MAGIC, 4));
}

use ioctl::*;

impl NvMap {
    /// Create a new instance of NvMap by opening `/dev/nvmap`.
    pub fn new() -> std::io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/nvmap")?;
        Ok(NvMap { file })
    }

    /// Create a new instance of NvMap from a file descriptor.
    pub fn new_from_raw_fd(raw_fd: RawFd) -> Self {
        NvMap {
            file: unsafe { File::from_raw_fd(raw_fd) },
        }
    }

    /// Get the file descriptor used.
    pub fn as_raw_fd(&self) -> RawFd {
        self.file.as_raw_fd()
    }

    /// Creates a new memory handle from a given size.
    pub fn create(&self, size: u32) -> NvMapResult<Handle> {
        let mut param = CreateHandle { size, handle: 0 };

        let res = unsafe { ioc_create(self.file.as_raw_fd(), &mut param) };
        if res.is_err() {
            Err(Errno::UnknownErrno)
        } else {
            let fd = self.get_fd(param.handle)?;

            let errno = res.unwrap();
            if errno == 0 {
                Ok(Handle::from_raw(param.handle, fd, size))
            } else {
                Err(Errno::from_i32(errno))
            }
        }
    }

    /// Creates a new memory handle by using another memory handle file descriptor.
    ///
    /// NOTE: The memory handle returned by this method will be referencing to the given file descriptor.
    pub fn create_from_fd(&self, fd: RawFd, size: u32) -> NvMapResult<Handle> {
        let mut param = CreateHandleFromFd { fd, handle: 0 };

        let res = unsafe { ioc_from_fd(self.file.as_raw_fd(), &mut param) };
        if res.is_err() {
            Err(Errno::UnknownErrno)
        } else {
            let errno = res.unwrap();
            if errno == 0 {
                Ok(Handle::from_raw(param.handle, fd, size))
            } else {
                Err(Errno::from_i32(errno))
            }
        }
    }

    /// Retrieve the file descriptor backing a memory handle.
    pub fn get_fd(&self, handle: RawHandle) -> NvMapResult<RawFd> {
        let mut param = HandleGetFd { fd: 0, handle };

        let res = unsafe { ioc_get_fd(self.file.as_raw_fd(), &mut param) };
        if res.is_err() {
            Err(Errno::UnknownErrno)
        } else {
            let errno = res.unwrap();
            if errno == 0 {
                Ok(param.fd)
            } else {
                Err(Errno::from_i32(errno))
            }
        }
    }

    /// Allocate GPU memory to the given memory handle.
    pub fn allocate(
        &self,
        handle: &Handle,
        heap_mask: HeapMask,
        flags: AllocationFlags,
        align: u32,
    ) -> NvMapResult<()> {
        let param = AllocateHandle {
            handle: handle.raw_handle,
            heap_mask: heap_mask.bits(),
            flags: flags.bits(),
            align,
        };

        let res = unsafe { ioc_allocate(self.file.as_raw_fd(), &param) };
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

    /// Map the GPU memory backing the given memory handle to the application address space.
    pub fn map(&self, handle: &mut Handle) -> NvMapResult<()> {
        if handle.addr().is_some() {
            return Ok(());
        }

        let mmap_res = unsafe {
            mmap(
                std::ptr::null_mut(),
                handle.size() as usize,
                ProtFlags::PROT_READ | ProtFlags::PROT_WRITE,
                MapFlags::MAP_SHARED,
                handle.fd,
                0,
            )
        }
        .or_else(|x| {
            let errno_opt = x.as_errno();
            if let Some(errno) = errno_opt {
                Err(errno)
            } else {
                Err(Errno::UnknownErrno)
            }
        })?;

        handle.mapped_address = Some(mmap_res as *mut u8);
        Ok(())
    }

    /// Unmap the backed GPU memory of a given memory handle from the application address space.
    pub fn unmap(&self, handle: &mut Handle) -> NvMapResult<()> {
        if let Some(addr) = handle.addr() {
            unsafe { munmap(addr as *mut _, handle.size as usize) }.or_else(|x| {
                let errno_opt = x.as_errno();
                if let Some(errno) = errno_opt {
                    Err(errno)
                } else {
                    Err(Errno::UnknownErrno)
                }
            })?;

            handle.mapped_address = None;
        }
        Ok(())
    }

    /// Operate cache maintenance of the backed memory of a given memory handle.
    fn cache_maintenance(
        &self,
        handle: &Handle,
        offset: u32,
        size: u32,
        operation: i32,
    ) -> NvMapResult<()> {
        if handle.addr().is_none() {
            return Ok(());
        }

        let mapped_address = handle.addr().unwrap();
        let param = HandleCacheMaintenance {
            address: mapped_address as u64 + u64::from(offset),
            handle: handle.raw_handle,
            length: size,
            operation,
        };

        let res = unsafe { ioc_cache(self.file.as_raw_fd(), &param) };
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

    /// Flush the cache of the backed memory of a given memory handle.
    pub fn writeback(&self, handle: &Handle, offset: u32, size: u32) -> NvMapResult<()> {
        self.cache_maintenance(handle, offset, size, CACHE_OPERATION_WRITE_BACK)
    }

    /// Invalidate the cache of the backed memory of a given memory handle.
    pub fn invalidate(&self, handle: &Handle, offset: u32, size: u32) -> NvMapResult<()> {
        self.cache_maintenance(handle, offset, size, CACHE_OPERATION_INVALIDATE)
    }

    /// Flush and invalidate the cache of the backed memory of a given memory handle.
    pub fn writeback_invalidate(&self, handle: &Handle, offset: u32, size: u32) -> NvMapResult<()> {
        self.cache_maintenance(handle, offset, size, CACHE_OPERATION_WRITE_BACK_INVALIDATE)
    }

    #[allow(clippy::cast_possible_wrap)]
    /// Free the memory handle and it's backed memory.
    pub fn free(&self, handle: Handle) -> NvMapResult<()> {
        let res = unsafe { ioc_free(self.file.as_raw_fd(), handle.raw_handle as i32) };
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
