use super::common::ReductionOperation;
use crate::utils::{Command, CommandStream, CommandSubmissionMode, SubChannelId};
use nvgpu::{GpuVirtualAddress, NvGpuResult};
use bitfield::BitRange;
use std::convert::TryInto;


#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum DependentQmdType {
    Queue,
    Grid
}

impl From<DependentQmdType> for u32 {
    fn from(mode: DependentQmdType) -> u32 {
        match mode {
            DependentQmdType::Queue => 0,
            DependentQmdType::Grid => 1
        }
    }
}

impl From<u32> for DependentQmdType {
    fn from(mode: u32) -> DependentQmdType {
        match mode {
            0 => DependentQmdType::Queue,
            1 => DependentQmdType::Grid,
            _ => unreachable!()
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum ReleaseMembarType {
    None,
    SysMembar,
}

impl From<ReleaseMembarType> for u32 {
    fn from(mode: ReleaseMembarType) -> u32 {
        match mode {
            ReleaseMembarType::None => 0,
            ReleaseMembarType::SysMembar => 1,
        }
    }
}

impl From<u32> for ReleaseMembarType {
    fn from(mode: u32) -> ReleaseMembarType {
        match mode {
            0 => ReleaseMembarType::None,
            1 => ReleaseMembarType::SysMembar,
            _ => unreachable!()
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum CwdMembarTypeL1 {
    None,
    SysMembar,
    Invalid,
    Membar,
}

impl From<CwdMembarTypeL1> for u32 {
    fn from(mode: CwdMembarTypeL1) -> u32 {
        match mode {
            CwdMembarTypeL1::None => 0,
            CwdMembarTypeL1::SysMembar => 1,
            CwdMembarTypeL1::Invalid => 2,
            CwdMembarTypeL1::Membar => 3
        }
    }
}

impl From<u32> for CwdMembarTypeL1 {
    fn from(mode: u32) -> CwdMembarTypeL1 {
        match mode {
            0 => CwdMembarTypeL1::None,
            1 => CwdMembarTypeL1::SysMembar,
            2 => CwdMembarTypeL1::Invalid,
            3 => CwdMembarTypeL1::Membar,
            _ => unreachable!()
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Fp32NanBehavior {
    Legacy,
    Fp64Compatible
}

impl From<Fp32NanBehavior> for u32 {
    fn from(mode: Fp32NanBehavior) -> u32 {
        match mode {
            Fp32NanBehavior::Legacy => 0,
            Fp32NanBehavior::Fp64Compatible => 1
        }
    }
}

impl From<u32> for Fp32NanBehavior {
    fn from(mode: u32) -> Fp32NanBehavior {
        match mode {
            0 => Fp32NanBehavior::Legacy,
            1 => Fp32NanBehavior::Fp64Compatible,
            _ => unreachable!()
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Fp32F2INanBehavior {
    PassZero,
    PassIndefinite
}

impl From<Fp32F2INanBehavior> for u32 {
    fn from(mode: Fp32F2INanBehavior) -> u32 {
        match mode {
            Fp32F2INanBehavior::PassZero => 0,
            Fp32F2INanBehavior::PassIndefinite => 1
        }
    }
}

impl From<u32> for Fp32F2INanBehavior {
    fn from(mode: u32) -> Fp32F2INanBehavior {
        match mode {
            0 => Fp32F2INanBehavior::PassZero,
            1 => Fp32F2INanBehavior::PassIndefinite,
            _ => unreachable!()
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum ApiVisibleCallLimit {
    Limit32,
    NoCheck
}

impl From<ApiVisibleCallLimit> for u32 {
    fn from(mode: ApiVisibleCallLimit) -> u32 {
        match mode {
            ApiVisibleCallLimit::Limit32 => 0,
            ApiVisibleCallLimit::NoCheck => 1
        }
    }
}

impl From<u32> for ApiVisibleCallLimit {
    fn from(mode: u32) -> ApiVisibleCallLimit {
        match mode {
            0 => ApiVisibleCallLimit::Limit32,
            1 => ApiVisibleCallLimit::NoCheck,
            _ => unreachable!()
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum SharedMemoryBankMapping {
    FourBytesPerBank,
    EightBytesPerBank
}

impl From<SharedMemoryBankMapping> for u32 {
    fn from(mode: SharedMemoryBankMapping) -> u32 {
        match mode {
            SharedMemoryBankMapping::FourBytesPerBank => 0,
            SharedMemoryBankMapping::EightBytesPerBank => 1
        }
    }
}

impl From<u32> for SharedMemoryBankMapping {
    fn from(mode: u32) -> SharedMemoryBankMapping {
        match mode {
            0 => SharedMemoryBankMapping::FourBytesPerBank,
            1 => SharedMemoryBankMapping::EightBytesPerBank,
            _ => unreachable!()
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum SamplerIndex {
    Independently,
    ViaHeaderIndex
}

impl From<SamplerIndex> for u32 {
    fn from(mode: SamplerIndex) -> u32 {
        match mode {
            SamplerIndex::Independently => 0,
            SamplerIndex::ViaHeaderIndex => 1
        }
    }
}

impl From<u32> for SamplerIndex {
    fn from(mode: u32) -> SamplerIndex {
        match mode {
            0 => SamplerIndex::Independently,
            1 => SamplerIndex::ViaHeaderIndex,
            _ => unreachable!()
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum Fp32NarrowInstruction {
    KeepDenorms,
    FlushDenorms
}

impl From<Fp32NarrowInstruction> for u32 {
    fn from(mode: Fp32NarrowInstruction) -> u32 {
        match mode {
            Fp32NarrowInstruction::KeepDenorms => 0,
            Fp32NarrowInstruction::FlushDenorms => 1
        }
    }
}

impl From<u32> for Fp32NarrowInstruction {
    fn from(mode: u32) -> Fp32NarrowInstruction {
        match mode {
            0 => Fp32NarrowInstruction::KeepDenorms,
            1 => Fp32NarrowInstruction::FlushDenorms,
            _ => unreachable!()
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum L1Configuration {
    DirectlyAddressableMemorySize16kb,
    DirectlyAddressableMemorySize32kb,
    DirectlyAddressableMemorySize48kb,
}

impl From<L1Configuration> for u32 {
    fn from(mode: L1Configuration) -> u32 {
        match mode {
            L1Configuration::DirectlyAddressableMemorySize16kb => 0,
            L1Configuration::DirectlyAddressableMemorySize32kb => 1,
            L1Configuration::DirectlyAddressableMemorySize48kb => 2
        }
    }
}

impl From<u32> for L1Configuration {
    fn from(mode: u32) -> L1Configuration {
        match mode {
            0 => L1Configuration::DirectlyAddressableMemorySize16kb,
            1 => L1Configuration::DirectlyAddressableMemorySize32kb,
            2 => L1Configuration::DirectlyAddressableMemorySize48kb,
            _ => unreachable!()
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
pub enum StructureSize {
    FourWords,
    OneWord,
}

impl From<StructureSize> for u32 {
    fn from(mode: StructureSize) -> u32 {
        match mode {
            StructureSize::FourWords => 0,
            StructureSize::OneWord => 1,
        }
    }
}

impl From<u32> for StructureSize {
    fn from(mode: u32) -> StructureSize {
        match mode {
            0 => StructureSize::FourWords,
            1 => StructureSize::OneWord,
            _ => unreachable!()
        }
    }
}

bitfield! {
    pub struct QueueMetaData17ConstantBuffer(u64);
    impl Debug;

    #[inline]
    pub u32, address_lower, set_address_lower: 31, 0;

    #[inline]
    pub u32, address_upper, set_address_upper: 39, 32;

    #[inline]
    pub u32, reserved_addr, set_reserved_addr: 45, 40;

    #[inline]
    pub invalidate, set_invalidate: 46;

    #[inline]
    pub u32, size, set_size: 63, 47;
}

#[repr(C)]
pub struct QueueMetaData17Release(pub [u32; 0x3]);

impl QueueMetaData17Release {
bitfield_fields! {
    #[inline]
    pub u32, address_lower, set_address_lower: 31, 0;

    #[inline]
    pub u32, address_upper, set_address_upper: 39, 32;

    #[inline]
    pub u32, from into ReductionOperation, reduction_op, set_reduction_op: 54, 52;

    #[inline]
    // TODO: enum this
    pub reduction_signed, set_reduction_signed: 56;

    #[inline]
    pub reduction_enable, set_reduction_enable: 58;

    #[inline]
    pub u32, from into StructureSize, structure_size, set_structure_size: 63, 63;

    #[inline]
    pub u32, payload, set_payload: 95, 64;
}
}

impl BitRange<u32> for QueueMetaData17Release {
    fn bit_range(&self, msb: usize, lsb: usize) -> u32 {
        let bit_len = core::mem::size_of::<u32>() * 8;
        let value_bit_len = core::mem::size_of::<u32>() * 8;

        let mut value = 0;

        for i in (lsb..=msb).rev() {
            value <<= 1;
            value |= ((self.0[i / bit_len] >> (i % bit_len)) & 1) as u32;
        }

        value << (value_bit_len - (msb - lsb + 1)) >> (value_bit_len - (msb - lsb + 1))
    }

    fn set_bit_range(&mut self, msb: usize, lsb: usize, value: u32) {
        let bit_len = core::mem::size_of::<u32>() * 8;

        let mut value = value;

        for i in lsb..=msb {
            self.0[i / bit_len] &= !(1 << (i % bit_len));
            self.0[i / bit_len] |= ((value & 1) as u32) << (i % bit_len);
            value >>= 1;
        }
    }
}

impl BitRange<u8> for QueueMetaData17Release {
    fn bit_range(&self, msb: usize, lsb: usize) -> u8 {
        let bit_len = core::mem::size_of::<u32>() * 8;
        let value_bit_len = core::mem::size_of::<u8>() * 8;

        let mut value = 0;

        for i in (lsb..=msb).rev() {
            value <<= 1;
            value |= ((self.0[i / bit_len] >> (i % bit_len)) & 1) as u8;
        }

        value << (value_bit_len - (msb - lsb + 1)) >> (value_bit_len - (msb - lsb + 1))
    }

    fn set_bit_range(&mut self, msb: usize, lsb: usize, value: u8) {
        let bit_len = core::mem::size_of::<u32>() * 8;

        let mut value = value;

        for i in lsb..=msb {
            self.0[i / bit_len] &= !(1 << (i % bit_len));
            self.0[i / bit_len] |= ((value & 1) as u32) << (i % bit_len);
            value >>= 1;
        }
    }
}


#[repr(C)]
pub struct QueueMetaData17(pub [u32; 0x40]);

impl QueueMetaData17 {
bitfield_fields! {
    #[inline]
    pub u32, outer_put, set_outer_put: 30, 0;

    #[inline]
    pub outer_overflow, set_outer_overflow: 31;

    #[inline]
    pub u32, outer_get, set_outer_get: 62, 32;

    #[inline]
    pub outer_sticky_overflow, set_outer_sticky_overflow: 63;

    #[inline]
    pub u32, inner_put, set_inner_put: 94, 64;

    #[inline]
    pub inner_overflow, set_inner_overflow: 95;

    #[inline]
    pub u32, inner_get, set_inner_get: 126, 96;

    #[inline]
    pub inner_sticky_overflow, set_inner_sticky_overflow: 127;

    #[inline]
    pub u32, qmd_reserved_a_a, set_qmd_reserved_a_a: 159, 128;

    #[inline]
    pub u32, dependent_qmd_pointer, set_dependent_qmd_pointer: 191, 160;

    #[inline]
    pub u8, qmd_group_id, set_qmd_group_id: 197, 192;

    #[inline]
    pub sm_global_caching_enable, set_sm_global_caching_enable: 198;

    #[inline]
    pub run_cta_in_one_sm_partition, set_run_cta_in_one_sm_partition: 199;

    #[inline]
    pub is_queue, set_is_queue: 200;

    #[inline]
    pub add_to_head_of_qmd_group_linked_list, set_add_to_head_of_qmd_group_linked_list: 201;

    #[inline]
    pub semaphore_release_enable0, set_semaphore_release_enable0: 202;

    #[inline]
    pub semaphore_release_enable1, set_semaphore_release_enable1: 203;
    
    #[inline]
    pub require_scheduling_pcas, set_require_scheduling_pcas: 204;

    #[inline]
    pub dependent_qmd_schedule_enable, set_dependent_qmd_schedule_enable: 205;

    #[inline]
    pub u32, from into DependentQmdType, dependent_qmd_type, set_dependent_qmd_type: 206, 206;

    #[inline]
    pub dependent_qmd_field_copy, set_dependent_qmd_field_copy: 207;

    #[inline]
    pub u32, qmd_reserved_b, set_qmd_reserved_b: 223, 208;

    #[inline]
    pub u32, circular_queue_size, set_circular_queue_size: 248, 224;

    #[inline]
    pub qmd_reserved_c, set_qmd_reserved_c: 249;

    #[inline]
    pub invalidate_texture_header_cache, set_invalidate_texture_header_cache: 250;

    #[inline]
    pub invalidate_texture_sampler_cache, set_invalidate_texture_sampler_cache: 251;

    #[inline]
    pub invalidate_texture_data_cache, set_invalidate_texture_data_cache: 252;

    #[inline]
    pub invalidate_shader_data_cache, set_invalidate_shader_data_cache: 253;

    #[inline]
    pub invalidate_instruction_cache, set_invalidate_instruction_cache: 254;

    #[inline]
    pub invalidate_shader_constant_cache, set_invalidate_shader_constant_cache: 255;

    #[inline]
    pub u32, program_offset, set_program_offset: 287, 256;

    #[inline]
    pub u32, circular_queue_addr_lower, set_circular_queue_addr_lower: 319, 288;

    #[inline]
    pub u32, circular_queue_addr_upper, set_circular_queue_addr_upper: 327, 320;

    #[inline]
    pub u32, qmd_reserved_d, set_qmd_reserved_d: 335, 328;

    #[inline]
    pub u32, circular_queue_entry_size, set_circular_queue_entry_size: 351, 336;

    #[inline]
    pub u32, cwd_reference_count_id, set_cwd_reference_count_id: 357, 352;

    #[inline]
    pub u32, cwd_reference_count_delta_minus_one, set_cwd_reference_count_delta_minus_one: 365, 358;

    #[inline]
    pub u32, from into ReleaseMembarType, release_membar_type, set_release_membar_type: 366, 366;

    #[inline]
    pub cwd_reference_count_incr_enable, set_cwd_reference_count_incr_enable: 367;

    #[inline]
    pub u32, from into CwdMembarTypeL1, cwd_membar_type, set_cwd_membar_type: 369, 368;

    #[inline]
    pub sequentially_run_ctas, set_sequentially_run_ctas: 370;

    #[inline]
    pub cwd_reference_count_decr_enable, set_cwd_reference_count_decr_enable: 371;

    #[inline]
    pub throttled, set_throttled: 372;

    // 372..376 missing?

    #[inline]
    pub u32, from into Fp32NanBehavior, fp32_nan_behavior, set_fp32_nan_behavior: 376, 376;

    #[inline]
    pub u32, from into Fp32F2INanBehavior, fp32_f2i_nan_behavior, set_fp32_f2i_nan_behavior: 377, 377;

    #[inline]
    pub u32, from into ApiVisibleCallLimit, api_visible_call_limit, set_api_visible_call_limit: 378, 378;

    #[inline]
    pub u32, from into SharedMemoryBankMapping, shared_memory_bank_mapping, set_shared_memory_bank_mapping: 379, 379;

    #[inline]
    pub u32, from into SamplerIndex, sampler_index, set_sampler_index: 382, 382;

    #[inline]
    pub u32, from into Fp32NarrowInstruction, fp32_narrow_instruction, set_fp32_narrow_instruction: 383, 383;

    #[inline]
    pub u32, cta_raster_width, set_cta_raster_width: 415, 384;

    #[inline]
    pub u32, cta_raster_height, set_cta_raster_height: 431, 416;

    #[inline]
    pub u32, cta_raster_depth, set_cta_raster_depth: 447, 432;

    #[inline]
    pub u32, cta_raster_width_resume, set_cta_raster_width_resume: 479, 448;

    #[inline]
    pub u32, cta_raster_height_resume, set_cta_raster_height_resume: 495, 480;

    #[inline]
    pub u32, cta_raster_depth_resume, set_cta_raster_depth_resume: 511, 496;

    #[inline]
    pub u32, queue_entries_per_cta_minus_one, set_queue_entries_per_cta_minus_one: 518, 512;

    #[inline]
    pub u32, coalesce_waiting_period, set_coalesce_waiting_period: 529, 522;

    // missing 529..544?

    #[inline]
    pub u32, shared_memory_size, set_shared_memory_size: 561, 544;

    #[inline]
    pub u32, qmd_reserved_g, set_qmd_reserved_g: 575, 562;

    #[inline]
    pub u32, qmd_version, set_qmd_version: 579, 576;

    #[inline]
    pub u32, qmd_major_version, set_qmd_major_version: 583, 580;

    #[inline]
    pub u32, qmd_reserved_h, set_qmd_reserved_h: 591, 584;

    #[inline]
    pub u32, cta_thread_dimension0, set_cta_thread_dimension0: 607, 592;

    #[inline]
    pub u32, cta_thread_dimension1, set_cta_thread_dimension1: 623, 608;

    #[inline]
    pub u32, cta_thread_dimension2, set_cta_thread_dimension2: 639, 624;

    #[inline]
    pub u8, constant_buffer_valid, set_constant_buffer_valid: 647, 640;

    #[inline]
    pub u32, qmd_reserved_i, set_qmd_reserved_i: 668, 648;

    #[inline]
    pub u32, from into L1Configuration, l1_configuration, set_l1_configuration: 671, 669;
    
    #[inline]
    pub u32, sm_disable_mask_lower, set_sm_disable_mask_lower: 703, 672;

    #[inline]
    pub u32, sm_disable_mask_upper, set_sm_disable_mask_upper: 735, 704;

    // NVB1C0_QMDV01_07_RELEASE0_ADDRESS_LOWER to NVB1C0_QMDV01_07_RELEASE1_PAYLOAD are handled manually.
    // NVB1C0_QMDV01_07_CONSTANT_BUFFER_ADDR_LOWER to NVB1C0_QMDV01_07_CONSTANT_BUFFER_SIZE are handled manually.

    #[inline]
    pub u32, shader_local_memory_low_size, set_shader_local_memory_low_size: 1463, 1440;

    #[inline]
    pub u32, qmd_reserved_n, set_qmd_reserved_n: 1466, 1464;

    #[inline]
    pub u32, barrier_count, set_barrier_count: 1471, 1467;

    #[inline]
    pub u32, shader_local_memory_high_size, set_shader_local_memory_high_size: 1495, 1472;

    #[inline]
    pub u32, register_count, set_register_count: 1503, 1496;

    #[inline]
    pub u32, shader_local_memory_crs_size, set_shader_local_memory_crs_size: 1527, 1504;

    #[inline]
    pub u32, sass_version, set_sass_version: 1535, 1528;

    #[inline]
    pub u32, hw_only_inner_get, set_hw_only_inner_get: 1566, 1536;

    #[inline]
    pub hw_only_require_scheduling_pcas, set_hw_only_require_scheduling_pcas: 1567;

    #[inline]
    pub u32, hw_only_inner_put, set_hw_only_inner_put: 1598, 1568;

    #[inline]
    pub hw_only_scg_type, set_hw_only_scg_type: 1599;

    #[inline]
    pub u32, hw_only_span_list_head_index, set_hw_only_span_list_head_index: 1629, 1600;

    #[inline]
    pub qmd_reserved_q, set_qmd_reserved_q: 1630;

    #[inline]
    pub hw_only_span_list_head_index_valid, set_hw_only_span_list_head_index_valid: 1631;

    #[inline]
    pub u32, hw_only_sked_next_qmd_pointer, set_hw_only_sked_next_qmd_pointer: 1663, 1632;

    #[inline]
    pub u32, qmd_spare_e, set_qmd_spare_e: 1695, 1664;

    #[inline]
    pub u32, qmd_spare_f, set_qmd_spare_f: 1727, 1696;

    #[inline]
    pub u32, qmd_spare_g, set_qmd_spare_g: 1759, 1728;

    #[inline]
    pub u32, qmd_spare_h, set_qmd_spare_h: 1791, 1760;

    #[inline]
    pub u32, qmd_spare_i, set_qmd_spare_i: 1823, 1792;

    #[inline]
    pub u32, qmd_spare_j, set_qmd_spare_j: 1855, 1824;

    #[inline]
    pub u32, qmd_spare_k, set_qmd_spare_k: 1887, 1856;

    #[inline]
    pub u32, qmd_spare_l, set_qmd_spare_l: 1919, 1888;

    #[inline]
    pub u32, qmd_spare_m, set_qmd_spare_m: 1951, 1920;

    #[inline]
    pub u32, qmd_spare_n, set_qmd_spare_n: 1983, 1952;

    #[inline]
    pub u32, debug_id_upper, set_debug_id_upper: 2015, 1984;

    #[inline]
    pub u32, debug_id_lower, set_debug_id_lower: 2047, 2016;
}
}

impl QueueMetaData17 {
    fn get_slice(&mut self, index: usize, size: usize) -> &mut [u32] {
        &mut self.0[index..index + size]
    }

    pub fn set_release(&mut self, index: usize, value: &QueueMetaData17Release) {
        if index > 1 {
            panic!("Invalid relase index {}", index);
        }

        let struc_size = core::mem::size_of::<QueueMetaData17Release>() / core::mem::size_of::<u32>();

        let output_slice = &mut self.0[0x17 + (index * struc_size)..0x17 + ((index + 1) * struc_size)];

        output_slice.copy_from_slice(&value.0[..]);
    }

    pub fn set_constant_buffer(&mut self, index: usize, value: &QueueMetaData17ConstantBuffer) {
        if index > 7 {
            panic!("Invalid constant buffer index {}", index);
        }

        let struc_size = core::mem::size_of::<QueueMetaData17ConstantBuffer>() / core::mem::size_of::<u32>();

        let output_slice = &mut self.0[0x1D + (index * struc_size)..0x1D + ((index + 1) * struc_size)];

        let bytes = value.0.to_le_bytes();

        output_slice[0] = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
        output_slice[1] = u32::from_le_bytes(bytes[4..8].try_into().unwrap())
    }
}

impl BitRange<u32> for QueueMetaData17 {
    fn bit_range(&self, msb: usize, lsb: usize) -> u32 {
        let bit_len = core::mem::size_of::<u32>() * 8;
        let value_bit_len = core::mem::size_of::<u32>() * 8;

        let mut value = 0;

        for i in (lsb..=msb).rev() {
            value <<= 1;
            value |= ((self.0[i / bit_len] >> (i % bit_len)) & 1) as u32;
        }

        value << (value_bit_len - (msb - lsb + 1)) >> (value_bit_len - (msb - lsb + 1))
    }

    fn set_bit_range(&mut self, msb: usize, lsb: usize, value: u32) {
        let bit_len = core::mem::size_of::<u32>() * 8;

        let mut value = value;

        for i in lsb..=msb {
            self.0[i / bit_len] &= !(1 << (i % bit_len));
            self.0[i / bit_len] |= ((value & 1) as u32) << (i % bit_len);
            value >>= 1;
        }
    }
}

impl BitRange<u16> for QueueMetaData17 {
    fn bit_range(&self, msb: usize, lsb: usize) -> u16 {
        let bit_len = core::mem::size_of::<u32>() * 8;
        let value_bit_len = core::mem::size_of::<u16>() * 8;

        let mut value = 0;

        for i in (lsb..=msb).rev() {
            value <<= 1;
            value |= ((self.0[i / bit_len] >> (i % bit_len)) & 1) as u16;
        }

        value << (value_bit_len - (msb - lsb + 1)) >> (value_bit_len - (msb - lsb + 1))
    }

    fn set_bit_range(&mut self, msb: usize, lsb: usize, value: u16) {
        let bit_len = core::mem::size_of::<u32>() * 8;

        let mut value = value;

        for i in lsb..=msb {
            self.0[i / bit_len] &= !(1 << (i % bit_len));
            self.0[i / bit_len] |= ((value & 1) as u32) << (i % bit_len);
            value >>= 1;
        }
    }
}

impl BitRange<u8> for QueueMetaData17 {
    fn bit_range(&self, msb: usize, lsb: usize) -> u8 {
        let bit_len = core::mem::size_of::<u32>() * 8;
        let value_bit_len = core::mem::size_of::<u8>() * 8;

        let mut value = 0;

        for i in (lsb..=msb).rev() {
            value <<= 1;
            value |= ((self.0[i / bit_len] >> (i % bit_len)) & 1) as u8;
        }

        value << (value_bit_len - (msb - lsb + 1)) >> (value_bit_len - (msb - lsb + 1))
    }

    fn set_bit_range(&mut self, msb: usize, lsb: usize, value: u8) {
        let bit_len = core::mem::size_of::<u32>() * 8;

        let mut value = value;

        for i in lsb..=msb {
            self.0[i / bit_len] &= !(1 << (i % bit_len));
            self.0[i / bit_len] |= ((value & 1) as u32) << (i % bit_len);
            value >>= 1;
        }
    }
}

pub fn memcpy_inline_host_to_device(
    command_stream: &mut CommandStream,
    dst: GpuVirtualAddress,
    data: &[u8],
) -> NvGpuResult<()> {
    debug_assert!(core::mem::size_of::<QueueMetaData17>() == 0x100);

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
