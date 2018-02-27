#![allow(unused)]

#![feature(attr_literals)]
#![feature(box_syntax)]
#![feature(inclusive_range_syntax)]
#![feature(match_default_bindings)]
#![feature(never_type)]
#![feature(nll)]
#![feature(trace_macros)]
#![feature(try_from)]

#![feature(plugin)]
#![plugin(phf_macros)]

#[macro_use]
extern crate failure_derive;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate phf;

extern crate itertools;
extern crate failure;



#[macro_use]
mod token;

mod ast;
mod lexer;
mod parser;
//pub mod compiler;
//pub mod codegen;
