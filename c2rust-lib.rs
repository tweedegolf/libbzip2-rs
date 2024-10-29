#![allow(dead_code)]
#![allow(mutable_transmutes)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(unused_assignments)]
#![allow(unused_mut)]
#![feature(extern_types)]
#![feature(linkage)]


extern crate libc;
pub mod blocksort;
pub mod bzlib;
pub mod compress;
pub mod crctable;
pub mod decompress;
pub mod huffman;
pub mod randtable;
