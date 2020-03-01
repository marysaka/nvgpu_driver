use std::fmt;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::marker::PhantomData;
use std::ops::Deref;
use std::ops::DerefMut;
use std::sync::Mutex;

use nvgpu::*;
use nvmap::*;

use super::{get_as, get_nvmap};

const PAGE_SIZE: u32 = 0x1000;

/// A Box but availaible to the GPU
pub struct GpuBox<T: Sized> {
    inner: GpuAllocated,
    phantom: PhantomData<T>,
}

impl<T: Sized> GpuBox<T> {
    pub fn new(x: T) -> GpuBox<T> {
        let inner =
            GpuAllocated::new(std::mem::size_of::<T>(), 0x20000).expect("Cannot allocate GpuBox!");

        let mut res = GpuBox {
            inner,
            phantom: PhantomData,
        };

        *res = x;

        // Flush inital data
        res.flush().expect("Cannot flush initial GpuBox data");

        res
    }

    pub fn unmap(&self) -> NvMapResult<()> {
        self.inner.unmap()
    }

    pub fn invalidate(&self) -> NvMapResult<()> {
        self.inner.invalidate()
    }

    pub fn flush(&self) -> NvMapResult<()> {
        self.inner.flush()
    }

    pub fn gpu_address(&self) -> GpuVirtualAddress {
        self.inner.gpu_address()
    }
}

impl<T: Sized> Deref for GpuBox<T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.inner.map().expect("Cannot map")
    }
}

impl<T: Sized> DerefMut for GpuBox<T> {
    fn deref_mut(&mut self) -> &mut T {
        self.inner.map_mut().expect("Cannot map_mut")
    }
}

pub struct GpuAllocated {
    handle: Mutex<Handle>,
    gpu_address: GpuVirtualAddress,
    user_size: usize,
}

impl Debug for GpuAllocated {
    /// Debug does not access reserved registers.
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        f.debug_struct("GpuAllocated")
            .field("handle", &self.handle)
            .field("gpu_address", &self.gpu_address)
            .finish()
    }
}

impl GpuAllocated {
    // TODO: kind
    pub fn new(user_size: usize, align: usize) -> NvGpuResult<Self> {
        let align = if align < PAGE_SIZE as usize {
            PAGE_SIZE
        } else {
            align as u32
        };

        let size = (user_size as u32 + (PAGE_SIZE - 1)) & !(PAGE_SIZE - 1);

        let nvmap = get_nvmap();
        let nvgpu_as = get_as();

        let nvmap_handle = nvmap.create(size)?;
        nvmap.allocate(
            &nvmap_handle,
            HeapMask::CARVEOUT_GENERIC,
            AllocationFlags::HANDLE_WRITE_COMBINE,
            align,
        )?;
        let gpu_address = nvgpu_as.map_buffer(&nvmap_handle, 0, PAGE_SIZE, 0)?;

        Ok(GpuAllocated::from_raw(nvmap_handle, gpu_address, user_size))
    }

    pub fn from_raw(handle: Handle, gpu_address: GpuVirtualAddress, user_size: usize) -> Self {
        GpuAllocated {
            handle: Mutex::new(handle),
            gpu_address,
            user_size,
        }
    }

    pub fn map<T: Sized>(&self) -> NvMapResult<&T> {
        let mut handle = self.handle.lock().unwrap();
        get_nvmap().map(&mut *handle)?;

        let mapped_address = handle.addr().expect("Handle address is null!");

        let ptr = mapped_address as *mut T;

        Ok(unsafe { ptr.as_mut().unwrap() })
    }

    pub fn map_mut<T: Sized>(&self) -> NvMapResult<&mut T> {
        let mut handle = self.handle.lock().unwrap();
        get_nvmap().map(&mut *handle)?;

        let mapped_address = handle.addr().expect("Handle address is null!");

        let ptr = mapped_address as *mut T;

        Ok(unsafe { ptr.as_mut().unwrap() })
    }

    pub fn map_array<T: Sized>(&self) -> NvMapResult<&[T]> {
        let mut handle = self.handle.lock().unwrap();
        get_nvmap().map(&mut *handle)?;

        let mapped_address = handle.addr().expect("Handle address is null!");

        let ptr = mapped_address as *mut T;

        Ok(unsafe { std::slice::from_raw_parts(ptr, self.user_size() / std::mem::size_of::<T>()) })
    }

    pub fn map_array_mut<T: Sized>(&self) -> NvMapResult<&mut [T]> {
        let mut handle = self.handle.lock().unwrap();
        get_nvmap().map(&mut *handle)?;

        let mapped_address = handle.addr().expect("Handle address is null!");

        let ptr = mapped_address as *mut T;

        Ok(unsafe {
            std::slice::from_raw_parts_mut(ptr, self.user_size() / std::mem::size_of::<T>())
        })
    }

    pub fn unmap(&self) -> NvMapResult<()> {
        let mut handle = self.handle.lock().unwrap();
        get_nvmap().unmap(&mut handle)
    }

    pub fn invalidate(&self) -> NvMapResult<()> {
        let handle = self.handle.lock().unwrap();
        get_nvmap().invalidate(&handle, 0, handle.size())
    }

    pub fn flush(&self) -> NvMapResult<()> {
        let handle = self.handle.lock().unwrap();
        get_nvmap().writeback_invalidate(&handle, 0, handle.size())
    }

    pub fn gpu_address(&self) -> GpuVirtualAddress {
        self.gpu_address
    }

    pub fn user_size(&self) -> usize {
        self.user_size
    }
}

impl Drop for GpuAllocated {
    fn drop(&mut self) {
        //println!("Dropping out of scope {:?}", self);

        self.unmap().expect("Cannot unmap from CPU side");

        let nvgpu_as = get_as();
        nvgpu_as
            .unmap_buffer(self.gpu_address())
            .expect("Cannot unmap GpuAllocated!");
    }
}
