use nvgpu::*;
use nvmap::*;

pub mod command_stream;
pub mod gpu_box;

pub use command_stream::*;
pub use gpu_box::*;

static mut NVMAP_INSTANCE: *mut NvMap = std::ptr::null_mut();
static mut NVAS_INSTANCE: *mut AddressSpace = std::ptr::null_mut();
static mut NVHOST_CTRL_INSTANCE: *mut NvHostGpuCtrl = std::ptr::null_mut();

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

pub fn initialize() -> std::io::Result<nvgpu::Channel> {
    init_nvhost_gpu_control()?;
    init_nvmap()?;
    init_address_space()?;

    let nvhost_gpu_ctrl = get_nvhost_gpu_ctrl();
    let nvmap = get_nvmap();
    let nvgpu_as = get_as();
    let nvtsg_channel = nvhost_gpu_ctrl.open_tsg()?;

    let nvgpu_channel = nvhost_gpu_ctrl.open_channel(-1, nvmap, nvgpu_as, Some(&nvtsg_channel))?;

    Ok(nvgpu_channel)
}

pub fn initialize_command_stream<'a>(
    channel: &'a nvgpu::Channel,
) -> NvGpuResult<CommandStream<'a>> {
    let mut command_stream = CommandStream::new(&channel);

    setup_channel(&mut command_stream)?;

    Ok(command_stream)
}

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
