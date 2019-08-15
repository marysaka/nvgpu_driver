//! Little test for nvmap
use nvmap::*;

#[allow(clippy::cast_ptr_alignment)]

fn main() -> NvMapResult<()> {
    let nvmap = NvMap::new().unwrap();

    let mut handle = nvmap.create(0x1000)?;
    println!("New handle: {:x}", handle.raw_handle);

    nvmap.allocate(
        &handle,
        HeapMask::CARVEOUT_GENERIC,
        AllocationFlags::HANDLE_WRITE_COMBINE,
        0x10,
    )?;

    let fd = nvmap.get_fd(handle.raw_handle)?;
    let mut handle_duplicate = nvmap.create_from_fd(fd, handle.size())?;

    nvmap.map(&mut handle)?;
    nvmap.map(&mut handle_duplicate)?;

    unsafe {
        let handle_addr = handle.addr().unwrap() as *mut u32;
        // Write the magic number
        *handle_addr = 0xCAFE_BABE;

        // Read the duplicate handle
        let handle_duplicate_addr = handle_duplicate.addr().unwrap() as *mut u32;
        println!(
            "The magic we all wanted to see: 0x{:X}",
            *handle_duplicate_addr
        );

        println!("Now cache maintenance!");
        println!("WRITEBACK");
        nvmap.writeback(&handle, 0, 4)?;
        println!("INVALIDATE");
        nvmap.invalidate(&handle, 0, 4)?;
        println!("WRITEBACK + INVALIDATE");
        nvmap.writeback_invalidate(&handle, 0, 4)?;
    }

    nvmap.unmap(&mut handle_duplicate)?;
    nvmap.unmap(&mut handle)?;
    nvmap.free(handle)?;

    Ok(())
}
