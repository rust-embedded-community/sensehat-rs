//! Registers for the HT221 humidity sensor
pub const REG_AV_CONF: u8 = 0x10;
pub const REG_CTRL1: u8 = 0x20;
pub const REG_STATUS: u8 = 0x27;
pub const REG_HUMIDITY_OUT_L: u8 = 0x28;
pub const REG_HUMIDITY_OUT_H: u8 = 0x29;
pub const REG_TEMP_OUT_L: u8 = 0x2a;
pub const REG_TEMP_OUT_H: u8 = 0x2b;
pub const REG_H0_H_2: u8 = 0x30;
pub const REG_H1_H_2: u8 = 0x31;
pub const REG_T0_C_8: u8 = 0x32;
pub const REG_T1_C_8: u8 = 0x33;
pub const REG_T1_T0: u8 = 0x35;
pub const REG_H0_T0_OUT: u8 = 0x36;
pub const REG_H1_T0_OUT: u8 = 0x3a;
pub const REG_T0_OUT: u8 = 0x3c;
pub const REG_T1_OUT: u8 = 0x3e;
