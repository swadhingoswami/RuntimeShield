use crate::core::error::{Error, Result};
use crate::platform::MemoryRegionReader;

type MachPortT = u32;
type MachVmAddressT = u64;
type MachVmSizeT = u64;
type KernReturnT = i32;
type VmProtT = i32;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct VmRegionSubmapInfo {
    protection: VmProtT,
    max_protection: VmProtT,
    inheritance: u32,
    offset: u32,
    user_tag: u32,
    ref_count: u32,
    shadow_depth: u32,
    external_pager: u32,
    share_mode: u8,
    is_submap: u32,
    behavior: i32,
    object_id: u32,
    user_wired_count: u16,
}

const VM_REGION_SUBMAP_INFO_COUNT: u32 =
    (std::mem::size_of::<VmRegionSubmapInfo>() / std::mem::size_of::<u32>()) as u32;

const KERN_SUCCESS: i32 = 0;
const VM_PROT_EXECUTE: VmProtT = 0x04;
const VM_PROT_WRITE: VmProtT = 0x02;

extern "C" {
    fn mach_task_self() -> MachPortT;
    fn mach_vm_region_recurse(
        target_task: MachPortT,
        address: *mut MachVmAddressT,
        size: *mut MachVmSizeT,
        nesting_depth: *mut u32,
        info: *mut VmRegionSubmapInfo,
        info_count: *mut u32,
    ) -> KernReturnT;

    fn mach_vm_read(
        target_task: MachPortT,
        address: MachVmAddressT,
        size: MachVmSizeT,
        data: *mut *mut libc::c_void,
        data_size: *mut MachVmSizeT,
    ) -> KernReturnT;

    fn vm_deallocate(
        target_task: MachPortT,
        address: MachVmAddressT,
        size: MachVmSizeT,
    ) -> KernReturnT;
}

#[derive(Clone)]
pub struct MacosMemory;

impl MacosMemory {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MacosMemory {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryRegionReader for MacosMemory {
    fn read_region(&self, address: usize, size: usize) -> Result<Vec<u8>> {
        let task = unsafe { mach_task_self() };
        let mut data: *mut libc::c_void = std::ptr::null_mut();
        let mut data_size: MachVmSizeT = 0;

        let kr = unsafe {
            mach_vm_read(
                task,
                address as MachVmAddressT,
                size as MachVmSizeT,
                &mut data,
                &mut data_size,
            )
        };

        if kr != KERN_SUCCESS {
            return Err(Error::Platform(format!(
                "mach_vm_read failed at {:#x} size {}: kern_return={}",
                address, size, kr
            )));
        }

        let slice = unsafe { std::slice::from_raw_parts(data as *const u8, data_size as usize) };
        let out = slice.to_vec();

        unsafe {
            vm_deallocate(task, data as MachVmAddressT, data_size);
        }

        Ok(out)
    }

    fn get_code_regions(&self) -> Result<Vec<(usize, usize)>> {
        let task = unsafe { mach_task_self() };
        let mut regions = Vec::new();
        let mut address: MachVmAddressT = 0;

        loop {
            let mut region_size: MachVmSizeT = 0;
            let mut info = std::mem::MaybeUninit::<VmRegionSubmapInfo>::uninit();
            let mut info_count = VM_REGION_SUBMAP_INFO_COUNT;
            let mut nesting_depth: u32 = 0;

            let kr = unsafe {
                mach_vm_region_recurse(
                    task,
                    &mut address,
                    &mut region_size,
                    &mut nesting_depth,
                    info.as_mut_ptr(),
                    &mut info_count,
                )
            };

            if kr != KERN_SUCCESS {
                break;
            }

            let info = unsafe { info.assume_init() };

            // Include if executable AND not writable
            if (info.protection & VM_PROT_EXECUTE) != 0 && (info.protection & VM_PROT_WRITE) == 0 {
                regions.push((address as usize, region_size as usize));
            }

            address = address.wrapping_add(region_size);
        }

        Ok(regions)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_regions() {
        let mem = MacosMemory::new();
        let result = mem.get_code_regions();
        assert!(
            result.is_ok(),
            "get_code_regions failed: {:?}",
            result.err()
        );
        let regions = result.unwrap();
        assert!(!regions.is_empty(), "should have at least one code region");
        for (_, size) in &regions {
            assert!(*size > 0, "region has zero size");
        }
    }

    #[test]
    fn test_read_own_code_region() {
        let mem = MacosMemory::new();
        let regions = mem.get_code_regions().unwrap();
        if let Some(&(addr, size)) = regions.first() {
            let n = size.min(64);
            let bytes = mem.read_region(addr, n).unwrap();
            assert!(!bytes.is_empty());
            assert_eq!(bytes.len(), n);
        }
    }
}
