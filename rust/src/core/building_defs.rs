#![allow(dead_code)]

pub mod building {
    pub const INTERNET: i32 = 0;
    pub const DATACENTER: i32 = 1;
    pub const CONVEYOR: i32 = 2;
    // Filter IDs 10-14
    pub const IP_FILTER: i32 = 10;
    pub const PORT_FILTER: i32 = 11;
    pub const LENGTH_FILTER: i32 = 12;
    pub const PROTOCOL_FILTER: i32 = 13;
    pub const CONTENT_FILTER: i32 = 14;

    pub const JUNCTION: i32 = 15;
    pub const RECYCLE_BIN: i32 = 16;
}
