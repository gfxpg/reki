#[derive(Default, Debug)]
pub struct KernelCodeObject {
    kernel_code_version_major: u32,
    kernel_code_version_minor: u32,
    machine_kind: u16,
    machine_version_major: u16,
    machine_version_minor: u16,
    machine_version_stepping: u16,

    kernel_code_entry_byte_offset: i64,

    kernel_code_prefetch_byte_offset: i64,
    kernel_code_prefetch_byte_size: u64,

    /* TODO: move to separate structs? */
    compute_pgm_resource_registers: u64,
    code_properties: u32,

    workitem_private_segment_byte_size: u32,
    workgroup_group_segment_byte_size: u32,
    gds_segment_byte_size: u32,

    kernarg_segment_byte_size: u64,

    workgroup_fbarrier_count: u32,

    wavefront_sgpr_count: u16,
    workitem_vgpr_count: u16,
    reserved_vgpr_first: u16,
    reserved_vgpr_count: u16,
    reserved_sgpr_first: u16,
    reserved_sgpr_count: u16,

    debug_wavefront_private_segment_offset_sgpr: u16,
    debug_private_segment_buffer_sgpr: u16,

    kernarg_segment_alignment: u8,
    group_segment_alignment: u8,
    private_segment_alignment: u8,

    wavefront_size: u8,

    call_convention: i32,
    runtime_loader_kernel_symbol: u64
}

use std::io;
use std::io::{Cursor, Seek, SeekFrom};
use byteorder::{LE, ReadBytesExt};
use std::convert::TryFrom;

impl <'a> TryFrom<&'a [u8]> for KernelCodeObject {
    type Error = io::Error;

    fn try_from(buf: &[u8]) -> Result<Self, io::Error> {
        if buf.len() != 256 {
            return Err(io::Error::from(io::ErrorKind::InvalidData));
        }
        let mut crs = Cursor::new(buf);
        let mut obj: KernelCodeObject = std::default::Default::default();

        obj.kernel_code_version_major = crs.read_u32::<LE>()?;
        obj.kernel_code_version_minor = crs.read_u32::<LE>()?;
        obj.machine_kind = crs.read_u16::<LE>()?;
        obj.machine_version_major = crs.read_u16::<LE>()?;
        obj.machine_version_minor = crs.read_u16::<LE>()?;
        obj.machine_version_stepping = crs.read_u16::<LE>()?;
        obj.kernel_code_entry_byte_offset = crs.read_i64::<LE>()?;
        obj.kernel_code_prefetch_byte_offset = crs.read_i64::<LE>()?;
        obj.kernel_code_prefetch_byte_size = crs.read_u64::<LE>()?;
        crs.seek(SeekFrom::Current(8))?; /* 8 bytes reserved */
        obj.compute_pgm_resource_registers = crs.read_u64::<LE>()?;
        obj.code_properties = crs.read_u32::<LE>()?;
        obj.workitem_private_segment_byte_size = crs.read_u32::<LE>()?;
        obj.workgroup_group_segment_byte_size = crs.read_u32::<LE>()?;
        obj.gds_segment_byte_size = crs.read_u32::<LE>()?;
        obj.kernarg_segment_byte_size = crs.read_u64::<LE>()?;
        obj.workgroup_fbarrier_count = crs.read_u32::<LE>()?;
        obj.wavefront_sgpr_count = crs.read_u16::<LE>()?;
        obj.workitem_vgpr_count = crs.read_u16::<LE>()?;
        obj.reserved_vgpr_first = crs.read_u16::<LE>()?;
        obj.reserved_vgpr_count = crs.read_u16::<LE>()?;
        obj.reserved_sgpr_first = crs.read_u16::<LE>()?;
        obj.reserved_sgpr_count = crs.read_u16::<LE>()?;
        obj.debug_wavefront_private_segment_offset_sgpr = crs.read_u16::<LE>()?;
        obj.debug_private_segment_buffer_sgpr = crs.read_u16::<LE>()?;
        obj.kernarg_segment_alignment = crs.read_u8()?;
        obj.group_segment_alignment = crs.read_u8()?;
        obj.private_segment_alignment = crs.read_u8()?;
        obj.wavefront_size = crs.read_u8()?;
        obj.call_convention = crs.read_i32::<LE>()?;
        crs.seek(SeekFrom::Current(12))?; /* 12 bytes reserved */
        obj.runtime_loader_kernel_symbol = crs.read_u64::<LE>()?;

        Ok(obj)
    }
}
