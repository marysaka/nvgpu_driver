use core::ops::{BitAnd, Not};
use num_traits::Num;
use nvgpu::*;
use nvmap::*;

pub mod command_stream;
pub mod gpu_box;

pub use command_stream::*;
pub use gpu_box::*;

static mut NVMAP_INSTANCE: *mut NvMap = std::ptr::null_mut();
static mut NVAS_INSTANCE: *mut AddressSpace = std::ptr::null_mut();
static mut NVHOST_CTRL_INSTANCE: *mut NvHostGpuCtrl = std::ptr::null_mut();

/// Align the address to the next alignment.
///
/// The given number should be a power of two to get coherent results!
///
/// # Panics
///
/// Panics on underflow if align is 0.
/// Panics on overflow if the expression `addr + (align - 1)` overflows.
pub fn align_up<T: Num + Not<Output = T> + BitAnd<Output = T> + Copy>(addr: T, align: T) -> T {
    align_down(addr + (align - T::one()), align)
}

/// Align the address to the previous alignment.
///
/// The given number should be a power of two to get coherent results!
///
/// # Panics
///
/// Panics on underflow if align is 0.
pub fn align_down<T: Num + Not<Output = T> + BitAnd<Output = T> + Copy>(addr: T, align: T) -> T {
    addr & !(align - T::one())
}

/// align_up, but checks if addr overflows
pub fn align_up_checked(addr: usize, align: usize) -> Option<usize> {
    match addr & (align - 1) {
        0 => Some(addr),
        _ => addr.checked_add(align - (addr % align)),
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

pub fn initialize() -> std::io::Result<(nvgpu::Channel, nvgpu::GpuCharacteristics)> {
    init_nvhost_gpu_control()?;
    init_nvmap()?;
    init_address_space()?;

    let nvhost_gpu_ctrl = get_nvhost_gpu_ctrl();
    let nvmap = get_nvmap();
    let nvgpu_as = get_as();
    let nvtsg_channel = nvhost_gpu_ctrl.open_tsg()?;

    let nvgpu_channel = nvhost_gpu_ctrl.open_channel(-1, nvmap, nvgpu_as, Some(&nvtsg_channel))?;

    Ok((nvgpu_channel, nvhost_gpu_ctrl.get_characteristics()?))
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

/// Creates a fake C-like enum, where all bit values are accepted.
///
/// This is mainly useful for FFI constructs. In C, an enum is allowed to take
/// any bit value, not just those defined in the enumeration. In Rust,
/// constructing an enum with a value outside the enumeration is UB. In order
/// to avoid this, we define our enum as a struct with associated variants.
#[macro_export]
macro_rules! enum_with_val {
    ($(#[$meta:meta])* $vis:vis struct $ident:ident($innervis:vis $ty:ty) {
        $($(#[$varmeta:meta])* $variant:ident = $num:expr),* $(,)*
    }) => {
        $(#[$meta])*
        #[repr(transparent)]
        $vis struct $ident($innervis $ty);
        impl $ident {
            $($(#[$varmeta])* $vis const $variant: $ident = $ident($num);)*
        }

        impl ::core::fmt::Debug for $ident {
            #[allow(unreachable_patterns)]
            fn fmt(&self, f: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                match self {
                    $(&$ident::$variant => write!(f, "{}::{}", stringify!($ident), stringify!($variant)),)*
                    &$ident(v) => write!(f, "UNKNOWN({})", v),
                }
            }
        }

        impl From<$ty> for $ident {
            fn from(data: $ty) -> $ident {
                $ident(data)
            }
        }

        impl From<$ident> for $ty {
            fn from(data: $ident) -> $ty {
                data.0
            }
        }
    }
}
