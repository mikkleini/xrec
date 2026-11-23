mod check_empty;
mod check_gaps;
mod checksum;
mod copy;
mod crc;
mod crc16;
mod crc32;
mod crc8;
mod erase;
mod fill;
mod help;
mod input;
mod move_memory;
mod output;
mod print_memory;
mod print_variables;
mod set_integer;
mod set_text;
mod set_vector;
mod sha;
mod sha2;
mod sha3;
mod silent;
mod start_address;
mod trim;
mod trim_begin;
mod trim_end;
mod version;
mod write_integer;
mod write_text;
mod write_vector;

use crate::args::Arguments;
use crate::context::Context;
use crate::Result;

pub(crate) fn dispatch(
    command: &str,
    args: &mut Arguments<'_>,
    context: &mut Context,
) -> Result<()> {
    match command {
        "-h" | "--help" => help::run(),
        "-v" | "--version" => version::run(),
        "-s" => silent::run(context),
        "-i" => input::run(args, context),
        "-o" => output::run(args, context),
        "-sa" => start_address::run(args, context),
        "-ce" => check_empty::run(args, context),
        "-cg" => check_gaps::run(args, context),
        "-f" => fill::run(args, context),
        "-e" => erase::run(args, context),
        "-tb" => trim_begin::run(args, context),
        "-te" => trim_end::run(args, context),
        "-c" => copy::run(args, context),
        "-m" => move_memory::run(args, context),
        "-st" => set_text::run(args, context),
        "-si" => set_integer::run(args, context),
        "-sv" => set_vector::run(args, context),
        "-csum" => checksum::run(args, context),
        "-crc8" => crc8::run(args, context),
        "-crc16" => crc16::run(args, context),
        "-crc32" => crc32::run(args, context),
        "-sha2" => sha2::run(args, context),
        "-sha3" => sha3::run(args, context),
        "-wt" => write_text::run(args, context),
        "-wi" => write_integer::run(args, context),
        "-wb" | "-wv" => write_vector::run(args, context),
        "-pm" => print_memory::run(context),
        "-pv" => print_variables::run(context),
        _ => Err(format!("unsupported command: {command}")),
    }
}

pub(crate) fn show_help() -> Result<()> {
    help::run()
}
