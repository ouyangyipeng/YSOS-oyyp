#![no_std]

use num_enum::FromPrimitive;

pub mod macros;

#[repr(usize)]
#[derive(Clone, Debug, FromPrimitive)]
pub enum Syscall {
    Read = 0,
    Write = 1,

    GetTime = 2,

    GetPid = 39,

    Sem = 40, // 0: new, 1: wait, 2: signal, 3: remove
    ListDir=42,
    OpenFile = 43,
    CloseFile = 44,
    
    Fork = 58,
    Spawn = 59,
    Exit = 60,
    WaitPid = 61,

    ListApp = 65531,
    Stat = 65532,
    Allocate = 65533,
    Deallocate = 65534,

    #[num_enum(default)]
    Unknown = 65535,
}
