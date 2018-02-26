#![allow(unused)]

#![feature(inclusive_range_syntax)]
#![feature(attr_literals)]
#![feature(trace_macros)]
#![feature(match_default_bindings)]
#![feature(box_syntax)]
#![feature(nll)]

#![feature(plugin)]
#![plugin(phf_macros)]

#[macro_use]
extern crate failure_derive;
extern crate itertools;

extern crate phf;
extern crate failure;



#[macro_use]
pub mod token;

pub mod ast;
pub mod lexer;
pub mod parser;
