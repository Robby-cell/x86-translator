use crate::prelude::{AssemblerOptions, FnSymbolResolver, assemble, assemble_with_options};
use crate::symbols;

mod basic;
mod control_flow;
mod data_processing;
mod directives;
mod memory;
mod resolvers;
mod stack;
