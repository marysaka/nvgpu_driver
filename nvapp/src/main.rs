use nvgpu::*;
use nvmap::*;

use std::fmt;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::marker::PhantomData;
use std::sync::Mutex;

use std::mem::ManuallyDrop;
use std::ops::Deref;
use std::ops::DerefMut;

const PAGE_SIZE: u32 = 0x1000;

static mut NVMAP_INSTANCE: *mut NvMap = std::ptr::null_mut();
static mut NVAS_INSTANCE: *mut AddressSpace = std::ptr::null_mut();
static mut NVHOST_CTRL_INSTANCE: *mut NvHostGpuCtrl = std::ptr::null_mut();

pub fn get_nvmap() -> &'static mut NvMap {
    unsafe { NVMAP_INSTANCE.as_mut().expect("NvMap not initialized") }
}

pub fn get_as() -> &'static mut AddressSpace {
    unsafe {
        NVAS_INSTANCE
            .as_mut()
            .expect("AddressSpace not initialized")
    }
}

pub fn get_nvhost_gpu_ctrl() -> &'static mut NvHostGpuCtrl {
    unsafe {
        NVHOST_CTRL_INSTANCE
            .as_mut()
            .expect("NvHostGpuCtrl not initialized")
    }
}

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

#[derive(Debug, PartialEq)]
pub enum CommandSubmissionMode {
    /// ?
    IncreasingOld,

    /// Tells PFIFO to read as much arguments as specified by argument count, while automatically incrementing the method value.
    /// This means that each argument will be written to a different method location.
    Increasing,

    /// ?
    NonIncreasingOld,

    /// Tells PFIFO to read as much arguments as specified by argument count.
    /// However, all arguments will be written to the same method location.
    NonIncreasing,

    /// Tells PFIFO to read inline data from bits 28-16 of the command word, thus eliminating the need to pass additional words for the arguments.
    Inline,

    /// Tells PFIFO to read as much arguments as specified by argument count and automatically increments the method value once only.
    IncreasingOnce,
}

pub struct Command {
    entry: GpFifoEntry,
    submission_mode: CommandSubmissionMode,
    arguments: Vec<u32>,
}

impl Command {
    pub fn new(method: u32, sub_channel: u32, submission_mode: CommandSubmissionMode) -> Self {
        let mut res = Command {
            entry: GpFifoEntry(0),
            submission_mode,
            arguments: Vec::new(),
        };

        res.entry.set_method(method);
        res.entry.set_sub_channel(sub_channel);

        let submission_mode_id = match res.submission_mode {
            CommandSubmissionMode::IncreasingOld => 0,
            CommandSubmissionMode::Increasing => 1,
            CommandSubmissionMode::NonIncreasingOld => 2,
            CommandSubmissionMode::NonIncreasing => 3,
            CommandSubmissionMode::Inline => 4,
            CommandSubmissionMode::IncreasingOnce => 5,
        };

        res.entry.set_submission_mode(submission_mode_id);

        res
    }

    pub fn new_inline(method: u32, sub_channel: u32, arguments: u32) -> Self {
        let mut res = Self::new(method, sub_channel, CommandSubmissionMode::Inline);
        res.entry.set_inline_arguments(arguments);

        res
    }

    pub fn push_argument(&mut self, argument: u32) {
        assert!(self.submission_mode != CommandSubmissionMode::Inline);
        self.arguments.push(argument);
    }

    pub fn push_address(&mut self, address: GpuVirtualAddress) {
        self.push_argument((address >> 32) as u32);
        self.push_argument(address as u32);
    }

    pub fn into_vec(mut self) -> Vec<u32> {
        let mut res = Vec::new();

        self.entry.set_argument_count(self.arguments.len() as u32);

        res.push(self.entry.0);
        res.append(&mut self.arguments);

        res
    }

    pub fn into_gpu_allocated(self) -> NvGpuResult<GpuAllocated> {
        let vec = self.into_vec();

        let res = GpuAllocated::new(vec.len() * std::mem::size_of::<u32>(), 0x20000)?;

        let arguments: &mut [u32] = res.map_array_mut()?;
        arguments.copy_from_slice(&vec[..]);

        res.flush()?;
        res.unmap()?;

        Ok(res)
    }
}

pub struct CommandStream<'a> {
    /// the inner implementation.
    fifo: ManuallyDrop<GpFifoQueue<'a>>,

    /// A Vec containing allocation to use in fifo.
    command_list: Vec<Command>,

    /// The previous command buffers kept alive to avoid being unmap by Drop during processing of the GPFIFO.
    in_process: ManuallyDrop<Vec<GpuAllocated>>,
}

impl<'a> Drop for CommandStream<'a> {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.fifo);
            ManuallyDrop::drop(&mut self.in_process);
        }
    }
}

impl<'a> CommandStream<'a> {
    pub fn new(channel: &'a Channel) -> Self {
        CommandStream {
            fifo: ManuallyDrop::new(GpFifoQueue::new(channel)),
            command_list: Vec::new(),
            in_process: ManuallyDrop::new(Vec::new()),
        }
    }

    pub fn push(&mut self, command: Command) -> NvGpuResult<()> {
        self.command_list.push(command);

        Ok(())
    }

    pub fn flush(&mut self) -> NvGpuResult<()> {
        let mut commands = Vec::new();

        for command in self.command_list.drain(..) {
            commands.append(&mut command.into_vec());
        }

        let commands_gpu = GpuAllocated::new(commands.len() * std::mem::size_of::<u32>(), 0x20000)?;

        let fifo_array: &mut [u32] = commands_gpu.map_array_mut()?;
        fifo_array.copy_from_slice(&commands[..]);

        commands_gpu.flush()?;
        commands_gpu.unmap()?;
        self.fifo.append(
            commands_gpu.gpu_address(),
            (commands_gpu.user_size() as u64) / 4,
            0,
        );

        self.in_process.push(commands_gpu);
        self.fifo.submit()?;

        Ok(())
    }

    pub fn wait_idle(&mut self) {
        self.fifo.wait_idle().unwrap();
    }
}

fn init_nvmap() -> std::io::Result<()> {
    let nvmap = NvMap::new()?;
    let nvmap_box = Box::new(nvmap);
    let nvmap_ref = Box::leak(nvmap_box);

    unsafe {
        NVMAP_INSTANCE = nvmap_ref as *mut NvMap;
    }

    Ok(())
}

fn init_address_space() -> std::io::Result<()> {
    let nvhost_gpu_ctrl = get_nvhost_gpu_ctrl();
    let address_space = nvhost_gpu_ctrl.allocate_address_space(0x10000, 0)?;
    let address_space_box = Box::new(address_space);
    let address_space_ref = Box::leak(address_space_box);

    unsafe {
        NVAS_INSTANCE = address_space_ref as *mut AddressSpace;
    }

    Ok(())
}

fn init_nvhost_gpu_control() -> std::io::Result<()> {
    let nvhost_ctrl = NvHostGpuCtrl::new()?;
    let nvhost_ctrl_box = Box::new(nvhost_ctrl);
    let nvhost_ctrl_ref = Box::leak(nvhost_ctrl_box);

    unsafe {
        NVHOST_CTRL_INSTANCE = nvhost_ctrl_ref as *mut NvHostGpuCtrl;
    }

    Ok(())
}

fn main() -> NvGpuResult<()> {
    init_nvhost_gpu_control().unwrap();
    init_nvmap().unwrap();
    init_address_space().unwrap();

    let nvhost_gpu_ctrl = get_nvhost_gpu_ctrl();
    let nvmap = get_nvmap();
    let nvgpu_as = get_as();
    let nvtsg_channel = nvhost_gpu_ctrl.open_tsg()?;

    let nvgpu_channel = nvhost_gpu_ctrl.open_channel(-1, nvmap, nvgpu_as, Some(&nvtsg_channel))?;

    let mut command_stream = CommandStream::new(&nvgpu_channel);

    let mut bind_channel_command = Command::new(0, 0, CommandSubmissionMode::Increasing);
    bind_channel_command.push_argument(0xB197);
    command_stream.push(bind_channel_command)?;

    let query_stats: GpuBox<[u64; 0x2]> = GpuBox::new([0xCAFE_BABE; 0x2]);
    println!("query_stats[1] initial: {:x}", query_stats[1]);

    let mut query_get_timestamp = Command::new(0x6c0, 0, CommandSubmissionMode::Increasing);
    query_get_timestamp.push_address(query_stats.gpu_address());
    query_get_timestamp.push_argument(0);
    query_get_timestamp.push_argument(0xf002);
    command_stream.push(query_get_timestamp)?;
    command_stream.flush()?;

    // Wait for the operations to be complete on the GPU side.
    command_stream.wait_idle();

    println!("query_stats[1] after query_get: {:x}", query_stats[1]);

    Ok(())
}
