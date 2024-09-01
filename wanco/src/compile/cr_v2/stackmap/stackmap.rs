/*
ref to https://llvm.org/docs/StackMaps.html#stack-map-format

Header {
  uint8  : Stack Map Version (current version is 3)
  uint8  : Reserved (expected to be 0)
  uint16 : Reserved (expected to be 0)
}
uint32 : NumFunctions
uint32 : NumConstants
uint32 : NumRecords
StkSizeRecord[NumFunctions] {
  uint64 : Function Address
  uint64 : Stack Size (or UINT64_MAX if not statically known)
  uint64 : Record Count
}
Constants[NumConstants] {
  uint64 : LargeConstant
}
StkMapRecord[NumRecords] {
  uint64 : PatchPoint ID
  uint32 : Instruction Offset
  uint16 : Reserved (record flags)
  uint16 : NumLocations
  Location[NumLocations] {
    uint8  : Register | Direct | Indirect | Constant | ConstantIndex
    uint8  : Reserved (expected to be 0)
    uint16 : Location Size
    uint16 : Dwarf RegNum
    uint16 : Reserved (expected to be 0)
    int32  : Offset or SmallConstant
  }
  uint32 : Padding (only if required to align to 8 byte)
  uint16 : Padding
  uint16 : NumLiveOuts
  LiveOuts[NumLiveOuts]
    uint16 : Dwarf RegNum
    uint8  : Reserved
    uint8  : Size in Bytes
  }
  uint32 : Padding (only if required to align to 8 byte)
}
*/

use crate::compile::cr_v2::stackmap::regs::AsStr;

use super::regs::Reg;
use anyhow::{anyhow, Result};
use nom::{
    error::{make_error, ErrorKind},
    multi::count,
    number::complete::{le_i32, le_u16, le_u32, le_u64, le_u8},
    sequence::Tuple,
    IResult,
};

pub fn parse(input: &[u8]) -> Result<Stackmap> {
    // https://stackoverflow.com/questions/55184864/nom-parser-borrow-checker-issue
    let map = parse_stackmap(input).map_err(|e| anyhow!(e.to_string()))?.1;
    Ok(map)
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Stackmap {
    pub header: Header,
    pub stack_size_records: Box<[StackSizeRecord]>,
    pub constants: Box<[u64]>,
    pub stackmap_records: Box<[StackMapRecord]>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Header {
    pub version: u8,
    pub reserved0: u8,
    pub reserved1: u16,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct StackSizeRecord {
    pub function_addr: u64,
    pub stack_size: u64,
    pub record_count: u64,
}
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StackMapRecord {
    pub patchpoint_id: u64,
    pub inst_offset: u32,
    pub locations: Box<[Location]>,
    pub live_outs: Box<[LiveOut]>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum LocationValue {
    /// Value is in a register,
    Register { reg: Reg },

    /// Frame index value.
    Direct { reg: Reg, offset: i32 },

    /// Spilled value.
    Indirect { reg: Reg, offset: i32 },

    /// Small constant.
    Constant { value: u32 },

    /// Large constant.
    ConstIndex { index: u32 },
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Location {
    pub value: LocationValue,
    pub size: u16,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct LiveOut {
    pub reg: Reg,
    pub size_in_bytes: u8,
}

/*
Header {
  ...
}
uint32 : NumFunctions
uint32 : NumConstants
uint32 : NumRecords
StkSizeRecord[NumFunctions] {
  ...
}
Constants[NumConstants] {
  ...
}
StkMapRecord[NumRecords] {
  ...
  uint32 : Padding (only if required to align to 8 byte)
}
*/
pub fn parse_stackmap(input: &[u8]) -> IResult<&[u8], Stackmap> {
    let (input, header) = parse_header(input)?;
    let (input, (num_functions, num_constants, num_records)) =
        (le_u32, le_u32, le_u32).parse(input)?;
    let (input, stack_size_records) =
        count(parse_stack_size_record, num_functions as usize)(input)?;
    let (input, constants) = count(le_u64, num_constants as usize)(input)?;
    let (input, stackmap_records) = count(parse_stackmap_record, num_records as usize)(input)?;

    Ok((
        input,
        Stackmap {
            header,
            stack_size_records: stack_size_records.into_boxed_slice(),
            constants: constants.into_boxed_slice(),
            stackmap_records: stackmap_records.into_boxed_slice(),
        },
    ))
}

fn parse_header(input: &[u8]) -> IResult<&[u8], Header> {
    let (input, version) = le_u8(input)?;
    let (input, reserved0) = le_u8(input)?;
    let (input, reserved1) = le_u16(input)?;
    Ok((
        input,
        Header {
            version,
            reserved0,
            reserved1,
        },
    ))
}

fn parse_stack_size_record(input: &[u8]) -> IResult<&[u8], StackSizeRecord> {
    let (input, function_addr) = le_u64(input)?;
    let (input, stack_size) = le_u64(input)?;
    let (input, record_count) = le_u64(input)?;
    Ok((
        input,
        StackSizeRecord {
            function_addr,
            stack_size,
            record_count,
        },
    ))
}

/*
StkMapRecord[NumRecords] {
  uint64 : PatchPoint ID
  uint32 : Instruction Offset
  uint16 : Reserved (record flags)
  uint16 : NumLocations
  Location[NumLocations] {
    ...
  }
  uint32 : Padding (only if required to align to 8 byte)
  uint16 : Padding
  uint16 : NumLiveOuts
  LiveOuts[NumLiveOuts]
    ...
  }
  uint32 : Padding (only if required to align to 8 byte)
}
*/
fn parse_stackmap_record(input: &[u8]) -> IResult<&[u8], StackMapRecord> {
    let (input, patchpoint_id) = le_u64(input)?;
    let (input, inst_offset) = le_u32(input)?;
    let (input, _) = le_u16(input)?; // reserved
    let (input, num_locations) = le_u16(input)?;
    let (input, locations) = count(parse_location, num_locations as usize)(input)?;
    // padding (only if required to align to 8 byte)
    let input = if input.as_ptr() as u64 % 8 != 0 {
        le_u32(input)?.0
    } else {
        input
    };
    let (input, _) = le_u16(input)?; // padding
    let (input, num_live_outs) = le_u16(input)?;
    let (input, live_outs) = count(parse_live_out, num_live_outs as usize)(input)?;
    // padding (only if required to align to 8 byte)
    let input = if input.as_ptr() as u64 % 8 != 0 {
        le_u32(input)?.0
    } else {
        input
    };

    Ok((
        input,
        StackMapRecord {
            patchpoint_id,
            inst_offset,
            locations: locations.into_boxed_slice(),
            live_outs: live_outs.into_boxed_slice(),
        },
    ))
}

/*
Location[NumLocations] {
  uint8  : Register | Direct | Indirect | Constant | ConstantIndex
  uint8  : Reserved (expected to be 0)
  uint16 : Location Size
  uint16 : Dwarf RegNum
  uint16 : Reserved (expected to be 0)
  int32  : Offset or SmallConstant
}
*/
fn parse_location(input: &[u8]) -> IResult<&[u8], Location> {
    let (input, kind) = le_u8(input)?;
    let (input, _) = le_u8(input)?; // reserved
    let (input, size) = le_u16(input)?;
    let (input, regnum) = le_u16(input)?;
    let (input, _) = le_u16(input)?; // reserved
    let (input, offset) = le_i32(input)?;

    let value = match kind {
        // Register.
        1 => LocationValue::Register {
            reg: Reg::from(regnum),
        },
        // Direct.
        2 => LocationValue::Direct {
            reg: Reg::from(regnum),
            offset,
        },
        // Indirect.
        3 => LocationValue::Indirect {
            reg: Reg::from(regnum),
            offset,
        },
        // Constant.
        4 => LocationValue::Constant {
            value: offset as u32,
        },
        // ConstIndex.
        5 => LocationValue::ConstIndex {
            index: offset as u32,
        },
        _ => {
            return Err(nom::Err::Error(make_error(input, ErrorKind::Tag)));
        }
    };

    Ok((input, Location { value, size }))
}

fn parse_live_out(input: &[u8]) -> IResult<&[u8], LiveOut> {
    let (input, regnum) = le_u16(input)?;
    let (input, _) = le_u8(input)?;
    let (input, size_in_bytes) = le_u8(input)?;
    Ok((
        input,
        LiveOut {
            reg: Reg::from(regnum),
            size_in_bytes,
        },
    ))
}

pub fn prettyprint(map: &Stackmap) {
    println!("Version: {}", map.header.version);
    println!("NumStackmapRecords: {}", map.stackmap_records.len());
    println!("NumConstants: {}", map.constants.len());
    println!("NumStackSizeRecords: {}", map.stack_size_records.len());
    for stackmap_record in map.stackmap_records.iter() {
        println!("Patchpoint ID: {}", stackmap_record.patchpoint_id);
        println!("- Instruction Offset: {}", stackmap_record.inst_offset);
        for location in stackmap_record.locations.iter() {
            match location.value {
                LocationValue::Register { reg } => {
                    println!(
                        "  - value: {} (size: {}, loc=reg)",
                        reg.as_str(),
                        location.size
                    );
                }
                LocationValue::Direct { reg, offset } => {
                    println!(
                        "  - value: {} + {} (size: {}, loc=direct)",
                        reg.as_str(),
                        offset,
                        location.size
                    );
                }
                LocationValue::Indirect { reg, offset } => {
                    println!(
                        "  - value: [{} {:+}] (size: {}, loc=indirect)",
                        reg.as_str(),
                        offset,
                        location.size
                    );
                }
                LocationValue::Constant { value } => {
                    println!("  - value: {} (size: {}, loc=const)", value, location.size);
                }
                LocationValue::ConstIndex { index } => {
                    println!(
                        "  - value: Constants[{}] (size: {}, loc=const_idx)",
                        index, location.size
                    );
                }
            }
        }
        for live_out in stackmap_record.live_outs.iter() {
            println!(
                "Live Out: {:?}, Size: {}",
                live_out.reg, live_out.size_in_bytes
            );
        }
    }
}
