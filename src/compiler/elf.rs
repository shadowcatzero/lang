use super::{program::Addr, LinkedProgram};

#[repr(C)]
pub struct ELF64Header {
    magic: u32,
    class: u8,
    endianness: u8,
    ei_version: u8,
    os_abi: u8,
    os_abi_ver: u8,
    pad: [u8; 7],
    ty: u16,
    machine: u16,
    e_version: u32,
    entry: u64,
    program_header_offset: u64,
    section_header_offset: u64,
    flags: u32,
    header_size: u16,
    program_header_entry_size: u16,
    program_header_num: u16,
    section_header_entry_size: u16,
    section_header_num: u16,
    section_header_str_idx: u16,
}

#[repr(C)]
pub struct ProgramHeader {
    ty: u32,
    flags: u32,
    offset: u64,
    vaddr: u64,
    paddr: u64,
    filesz: u64,
    memsz: u64,
    align: u64,
}

#[repr(C)]
pub struct SectionHeader {
    name_idx: u32,
    ty: u32,
    flags: u64,
    addr: u64,
    offset: u64,
    size: u64,
    link: u32,
    info: u32,
    addr_align: u64,
    entry_size: u64,
}

// this is currently specialized for riscv64; obviously add params later
pub fn create(program: &[u8], start_offset: Addr) -> Vec<u8> {
    let addr_start = 0x1000;
    let page_size = 0x1000;
    // I don't know if I have to add addr_start here, idk how it maps the memory
    let program_size = std::mem::size_of_val(program) as u64 + addr_start;
    let program_header = ProgramHeader {
        ty: 0x1,      // LOAD
        flags: 0b101, // executable, readable
        offset: 0x0,
        vaddr: addr_start,
        paddr: addr_start,
        filesz: program_size,
        memsz: program_size,
        align: page_size,
    };
    let header_len = (size_of::<ELF64Header>() + size_of::<ProgramHeader>()) as u64;
    let program_pos = header_len;
    let header = ELF64Header {
        magic: 0x7f_45_4c_46u32.swap_bytes(),
        class: 0x2,      // 64 bit
        endianness: 0x1, // little endian
        ei_version: 0x1,
        os_abi: 0x0, // system-v
        os_abi_ver: 0x0,
        pad: [0x0; 7],
        ty: 0x2,       // executable
        machine: 0xf3, // risc-v
        e_version: 0x1,
        entry: addr_start + program_pos + start_offset.val(),
        program_header_offset: size_of::<ELF64Header>() as u64,
        section_header_offset: 0x0,
        // C ABI (16 bit instruction align) + double precision floats
        flags: 0x1 | 0x4,
        header_size: size_of::<ELF64Header>() as u16,
        program_header_entry_size: size_of::<ProgramHeader>() as u16,
        program_header_num: 0x1,
        section_header_entry_size: size_of::<SectionHeader>() as u16,
        section_header_num: 0x0,
        section_header_str_idx: 0x0,
    };
    let mut bytes: Vec<u8> = Vec::new();
    unsafe {
        bytes.extend(as_u8_slice(&header));
        bytes.extend(as_u8_slice(&program_header));
        bytes.extend(program);
    }
    bytes
}

unsafe fn as_u8_slice<T: Sized>(p: &T) -> &[u8] {
    core::slice::from_raw_parts((p as *const T) as *const u8, size_of::<T>())
}

impl LinkedProgram {
    pub fn to_elf(&self) -> Vec<u8> {
        create(&self.code, self.start.expect("no start found"))
    }
}
