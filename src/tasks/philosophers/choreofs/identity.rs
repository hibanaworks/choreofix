//! Logical roles for the dining protocol and its observations.
pub const SESSION: u32 = 19;
pub const AGENTS: [&str; 7] = [
    "PHILOSOPHER_A",
    "PHILOSOPHER_B",
    "PHILOSOPHER_C",
    "PHILOSOPHER_D",
    "PHILOSOPHER_E",
    "PHILOSOPHER_F",
    "PHILOSOPHER_G",
];
pub mod roles {
    pub const A: u8 = 0;
    pub const FORK_A: u8 = 7;
    pub const A_WASI: u8 = 14;
    pub const A_IO: u8 = 21;
    pub const FORK_A_RESOURCE: u8 = 28;
    pub const A_THINK_FILE: u8 = 35;
    pub const THINK_A: u8 = 42;
    pub const A_EAT_FILE: u8 = 49;
    pub const B: u8 = 1;
    pub const FORK_B: u8 = 8;
    pub const B_WASI: u8 = 15;
    pub const B_IO: u8 = 22;
    pub const FORK_B_RESOURCE: u8 = 29;
    pub const B_THINK_FILE: u8 = 36;
    pub const THINK_B: u8 = 43;
    pub const B_EAT_FILE: u8 = 50;
    pub const C: u8 = 2;
    pub const FORK_C: u8 = 9;
    pub const C_WASI: u8 = 16;
    pub const C_IO: u8 = 23;
    pub const FORK_C_RESOURCE: u8 = 30;
    pub const C_THINK_FILE: u8 = 37;
    pub const THINK_C: u8 = 44;
    pub const C_EAT_FILE: u8 = 51;
    pub const D: u8 = 3;
    pub const FORK_D: u8 = 10;
    pub const D_WASI: u8 = 17;
    pub const D_IO: u8 = 24;
    pub const FORK_D_RESOURCE: u8 = 31;
    pub const D_THINK_FILE: u8 = 38;
    pub const THINK_D: u8 = 45;
    pub const D_EAT_FILE: u8 = 52;
    pub const E: u8 = 4;
    pub const FORK_E: u8 = 11;
    pub const E_WASI: u8 = 18;
    pub const E_IO: u8 = 25;
    pub const FORK_E_RESOURCE: u8 = 32;
    pub const E_THINK_FILE: u8 = 39;
    pub const THINK_E: u8 = 46;
    pub const E_EAT_FILE: u8 = 53;
    pub const F: u8 = 5;
    pub const FORK_F: u8 = 12;
    pub const F_WASI: u8 = 19;
    pub const F_IO: u8 = 26;
    pub const FORK_F_RESOURCE: u8 = 33;
    pub const F_THINK_FILE: u8 = 40;
    pub const THINK_F: u8 = 47;
    pub const F_EAT_FILE: u8 = 54;
    pub const G: u8 = 6;
    pub const FORK_G: u8 = 13;
    pub const G_WASI: u8 = 20;
    pub const G_IO: u8 = 27;
    pub const FORK_G_RESOURCE: u8 = 34;
    pub const G_THINK_FILE: u8 = 41;
    pub const THINK_G: u8 = 48;
    pub const G_EAT_FILE: u8 = 55;
}
