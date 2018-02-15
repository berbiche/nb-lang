#![allow(unused)]
#![feature(inclusive_range_syntax)]
//#![feature(trace_macros)]
//trace_macros!(true);

extern crate failure;
#[macro_use]
extern crate failure_derive;

pub mod token;
pub mod lexer;
