#[derive(Default, Debug)]
pub struct KernelArg {
    pub name: String,
    pub size: u32,
    pub offset: u32
}

pub fn extract_kernel_args(section_note: &Vec<u8>) -> Vec<KernelArg> {
    let cl_note: Vec<u8> = section_note
        .iter()
        .skip_while(|&&c| c != '\n' as u8)
        .filter(|&&c| c != 0)
        .map(|c| c.to_owned()).collect();
    
    let metadata = String::from_utf8_lossy(cl_note.as_slice());
    let args_raw: Vec<Vec<String>> = metadata
        .lines()
        .skip_while(|l| !l.starts_with("    Args:")).skip(1)
        .take_while(|l| !l.starts_with("    CodeProps:"))
        .fold(Vec::new(), |mut args, l| {
            if l.starts_with("      - ") {
                args.push(vec![l[8..].replace(" ", "")])
            }
            else {
                args.last_mut().unwrap().push(l.replace(" ", ""))
            }
            args
        });

    let mut offset = 0;

    args_raw
        .into_iter()
        .map(|args| {
            let name = args.iter().find(|e| e.starts_with("Name")).map(|e| &e[5..])
                .or(args.iter().find(|e| e.starts_with("ValueKind")).map(|e| &e[10..]))
                .unwrap().to_string();
            let size = args.iter().find(|e| e.starts_with("Size")).unwrap()[5..]
                .parse::<u32>().unwrap();
            let alignment = args.iter().find(|e| e.starts_with("Align")).unwrap()[6..]
                .parse::<u32>().unwrap();
            offset += offset % alignment;
            offset += size;
            KernelArg { name, size, offset: offset - size }
        })
        .collect()
}

#[derive(Default, Debug)]
pub struct KernelCode {
    pub kernel_code_version_major: u32,
    pub kernel_code_version_minor: u32,
    pub machine_kind: u16,
    pub machine_version_major: u16,
    pub machine_version_minor: u16,
    pub machine_version_stepping: u16,
    pub kernel_code_entry_byte_offset: i64,
    pub kernel_code_prefetch_byte_offset: i64,
    pub kernel_code_prefetch_byte_size: u64,
    pub workitem_private_segment_byte_size: u32,
    pub workgroup_group_segment_byte_size: u32,
    pub gds_segment_byte_size: u32,
    pub kernarg_segment_byte_size: u64,
    pub workgroup_fbarrier_count: u32,
    pub wavefront_sgpr_count: u16,
    pub workitem_vgpr_count: u16,
    pub reserved_vgpr_first: u16,
    pub reserved_vgpr_count: u16,
    pub reserved_sgpr_first: u16,
    pub reserved_sgpr_count: u16,
    pub debug_wavefront_private_segment_offset_sgpr: u16,
    pub debug_private_segment_buffer_sgpr: u16,
    pub kernarg_segment_alignment: u8,
    pub group_segment_alignment: u8,
    pub private_segment_alignment: u8,
    pub wavefront_size: u8,
    pub call_convention: i32,
    pub runtime_loader_kernel_symbol: u64,

    pub pgm_props: PgmProperties,
    pub code_props: CodeProperties
}

use std::io;
use std::io::{Cursor, Seek, SeekFrom};
use byteorder::{LE, ReadBytesExt};
use std::convert::TryFrom;

// https://llvm.org/docs/AMDGPUUsage.html#amdgpu-amdhsa-system-vgpr-work-item-id-enumeration-values-table
#[derive(Debug)]
pub enum VGPRWorkItemId {
    X,
    XY,
    XYZ
}

impl Default for VGPRWorkItemId {
    fn default() -> Self { VGPRWorkItemId::X }
}

impl From<u8> for VGPRWorkItemId {
    fn from(val: u8) -> Self {
        match val {
            0 => VGPRWorkItemId::X,
            1 => VGPRWorkItemId::XY,
            _ => VGPRWorkItemId::XYZ
        }
    }
}

// https://llvm.org/docs/AMDGPUUsage.html#amdgpu-amdhsa-floating-point-rounding-mode-enumeration-values-table
#[derive(Debug)]
pub enum FPRoundMode {
    NearEven,
    PlusInfinity,
    MinusInfinity,
    Zero
}

impl Default for FPRoundMode {
    fn default() -> Self { FPRoundMode::NearEven }
}

impl From<u8> for FPRoundMode {
    fn from(val: u8) -> Self {
        match val {
            0 => FPRoundMode::NearEven,
            1 => FPRoundMode::PlusInfinity,
            2 => FPRoundMode::MinusInfinity,
            _ => FPRoundMode::Zero
        }
    }
}

// https://llvm.org/docs/AMDGPUUsage.html#amdgpu-amdhsa-floating-point-denorm-mode-enumeration-values-table
#[derive(Debug)]
pub enum FPDenormMode {
    FlushSrcDst,
    FlushDst,
    FlushSrc,
    FlushNone
}

impl Default for FPDenormMode {
    fn default() -> Self { FPDenormMode::FlushSrcDst }
}

impl From<u8> for FPDenormMode {
    fn from(val: u8) -> Self {
        match val {
            0 => FPDenormMode::FlushSrcDst,
            1 => FPDenormMode::FlushDst,
            2 => FPDenormMode::FlushSrc,
            _ => FPDenormMode::FlushNone
        }
    }
}

#[derive(Default, Debug)]
pub struct PgmProperties {
    /* PGM_RSRC1 */
    pub granulated_workitem_vgpr_count: u8,
    pub granulated_wavefront_sgpr_count: u8,
    pub priority: u8,
    pub float_round_mode_32: FPRoundMode,
    pub float_round_mode_16_64: FPRoundMode,
    pub float_denorm_mode_32: FPDenormMode,
    pub float_denorm_mode_16_64: FPDenormMode,
    pub is_priv: bool,
    pub enable_dx10_clamp: bool,
    pub debug_mode: bool,
    pub enable_ieee_mode: bool,
    pub bulky: bool,
    pub cdbg_user: bool,
    pub fp16_ovfl: bool,
    /* PGM_RSRC2 */
    pub enable_sgpr_private_segment_wavefront_offset: bool,
    pub user_sgpr_count: u8,
    pub enable_trap_handler: bool,
    pub enable_sgpr_workgroup_id_x: bool,
    pub enable_sgpr_workgroup_id_y: bool,
    pub enable_sgpr_workgroup_id_z: bool,
    pub enable_sgpr_workgroup_info: bool,
    pub enable_vgpr_workitem_id: VGPRWorkItemId,
    pub enable_exception_address_watch: bool,
    pub enable_exception_memory: bool,
    pub granulated_lds_size: u8,
    pub enable_exception_ieee_754_fp_invalid_operation: bool,
    pub enable_exception_fp_denormal_source: bool,
    pub enable_exception_ieee_754_fp_division_by_zero: bool,
    pub enable_exception_ieee_754_fp_overflow: bool,
    pub enable_exception_ieee_754_fp_underflow: bool,
    pub enable_exception_ieee_754_fp_inexact: bool,
    pub enable_exception_int_divide_by_zero: bool
}

#[derive(Default, Debug)]
pub struct CodeProperties {
  pub enable_sgpr_private_segment_buffer: bool,
  pub enable_sgpr_dispatch_ptr: bool,
  pub enable_sgpr_queue_ptr: bool,
  pub enable_sgpr_kernarg_segment_ptr: bool,
  pub enable_sgpr_dispatch_id: bool,
  pub enable_sgpr_flat_scratch_init: bool,
  pub enable_sgpr_private_segment_size: bool,
  pub enable_sgpr_grid_workgroup_count_x: bool,
  pub enable_sgpr_grid_workgroup_count_y: bool,
  pub enable_sgpr_grid_workgroup_count_z: bool,
  pub enable_ordered_append_gds: bool,
  pub private_element_size: u8,
  pub is_ptr64: bool,
  pub is_dynamic_callstack: bool,
  pub is_debug_supported: bool,
  pub is_xnack_supported: bool
}

macro_rules! extract_bitfields {
    ([$source:expr => $dest:expr] { $($name:ident: bool at bit $shift:expr),* }) => {
        $(
            $dest.$name = (($source & (1 << $shift)) >> $shift) != 0;
        )*
    };
    ([$source:expr => $dest:expr] { $($name:ident: $type:ty, from bit $shift:expr, width $width:expr),* }) => {
        $(
            $dest.$name = (($source & (((1 << $width) - 1) << $shift)) >> $shift) as $type;
        )*
    }
}

macro_rules! get_bitfield {
    ($source:expr, from bit $shift:expr, width $width:expr) => {
        ($source & (((1 << $width) - 1) << $shift)) >> $shift;
    }
}

impl <'a> TryFrom<&'a [u8]> for KernelCode {
    type Error = io::Error;

    fn try_from(buf: &[u8]) -> Result<Self, io::Error> {
        if buf.len() != 256 {
            return Err(io::Error::from(io::ErrorKind::InvalidData));
        }
        let mut crs = Cursor::new(buf);
        let mut obj: KernelCode = Default::default();

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
        let compute_pgm_resource_registers = crs.read_u64::<LE>()?;
        let code_properties = crs.read_u32::<LE>()?;
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
    
        extract_bitfields!(
            [compute_pgm_resource_registers => obj.pgm_props] {
                is_priv: bool at bit 20,
                enable_dx10_clamp: bool at bit 21,
                debug_mode: bool at bit 22,
                enable_ieee_mode: bool at bit 23,
                bulky: bool at bit 24,
                cdbg_user: bool at bit 25,
                fp16_ovfl: bool at bit 26,

                enable_sgpr_private_segment_wavefront_offset: bool at bit 32 + 0,
                enable_trap_handler: bool at bit 32 + 6,
                enable_sgpr_workgroup_id_x: bool at bit 32 + 7,
                enable_sgpr_workgroup_id_y: bool at bit 32 + 8,
                enable_sgpr_workgroup_id_z: bool at bit 32 + 9,
                enable_sgpr_workgroup_info: bool at bit 32 + 10,
                enable_exception_address_watch: bool at bit 32 + 13,
                enable_exception_memory: bool at bit 32 + 14,
                enable_exception_ieee_754_fp_invalid_operation: bool at bit 32 + 24,
                enable_exception_fp_denormal_source: bool at bit 32 + 25,
                enable_exception_ieee_754_fp_division_by_zero: bool at bit 32 + 26,
                enable_exception_ieee_754_fp_overflow: bool at bit 32 + 27,
                enable_exception_ieee_754_fp_underflow: bool at bit 32 + 28,
                enable_exception_ieee_754_fp_inexact: bool at bit 32 + 29,
                enable_exception_int_divide_by_zero: bool at bit 32 + 30
            }
        );
        extract_bitfields!(
            [compute_pgm_resource_registers => obj.pgm_props] {
                granulated_workitem_vgpr_count: u8, from bit 0, width 6,
                granulated_wavefront_sgpr_count: u8, from bit 6, width 4,
                priority: u8, from bit 10, width 2,

                user_sgpr_count: u8, from bit 32 + 1, width 5,
                granulated_lds_size: u8, from bit 32 + 15, width 9
            }
        );
        obj.pgm_props.float_round_mode_32 = FPRoundMode::from(
            get_bitfield!(compute_pgm_resource_registers, from bit 12, width 2) as u8);
        obj.pgm_props.float_round_mode_16_64 = FPRoundMode::from(
            get_bitfield!(compute_pgm_resource_registers, from bit 14, width 2) as u8);
        obj.pgm_props.float_denorm_mode_32 = FPDenormMode::from(
            get_bitfield!(compute_pgm_resource_registers, from bit 16, width 2) as u8);
        obj.pgm_props.float_denorm_mode_16_64 = FPDenormMode::from(
            get_bitfield!(compute_pgm_resource_registers, from bit 18, width 2) as u8);
        obj.pgm_props.enable_vgpr_workitem_id = VGPRWorkItemId::from(
            get_bitfield!(compute_pgm_resource_registers, from bit 32 + 11, width 2) as u8);

        extract_bitfields!(
            [code_properties => obj.code_props] {
                enable_sgpr_private_segment_buffer: bool at bit 0,
                enable_sgpr_dispatch_ptr: bool at bit 1,
                enable_sgpr_queue_ptr: bool at bit 2,
                enable_sgpr_kernarg_segment_ptr: bool at bit 3,
                enable_sgpr_dispatch_id: bool at bit 4,
                enable_sgpr_flat_scratch_init: bool at bit 5,
                enable_sgpr_private_segment_size: bool at bit 6,
                enable_sgpr_grid_workgroup_count_x: bool at bit 7,
                enable_sgpr_grid_workgroup_count_y: bool at bit 8,
                enable_sgpr_grid_workgroup_count_z: bool at bit 9,
                enable_ordered_append_gds: bool at bit 16,
                is_ptr64: bool at bit 19,
                is_dynamic_callstack: bool at bit 20,
                is_debug_supported: bool at bit 21,
                is_xnack_supported: bool at bit 22
            }
        );
        extract_bitfields!(
            [code_properties => obj.code_props] {
                private_element_size: u8, from bit 17, width 2
            }
        );

        Ok(obj)
    }
}
