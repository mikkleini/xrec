use crate::args::Arguments;
use crate::checksum::crc;
use crate::context::Context;
use crate::value::{expression, variable_name, Value};
use crate::Result;

pub(crate) fn run(args: &mut Arguments<'_>, context: &mut Context, width: u32) -> Result<()> {
    let start = expression(&args.required()?, &context.vars)?;
    let len = expression(&args.required()?, &context.vars)?;
    let poly = expression(&args.required()?, &context.vars)?;
    let init = expression(&args.required()?, &context.vars)?;
    let xor_out = expression(&args.required()?, &context.vars)?;
    let mut refin = false;
    let mut refout = false;
    loop {
        if args.peek_next_is_command() {
            break;
        }
        match args.peek() {
            Some(flag) if flag.eq_ignore_ascii_case("rin") => {
                refin = true;
                args.advance();
            }
            Some(flag) if flag.eq_ignore_ascii_case("rout") => {
                refout = true;
                args.advance();
            }
            _ => break,
        }
    }
    let name = args.required()?;
    variable_name(&name)?;
    if context.vars.contains_key(&name) {
        return Err(format!("variable already exists: {name}"));
    }
    let limit = (1u64 << width) - 1;
    if poly > limit || init > limit || xor_out > limit {
        return Err(format!("CRC parameters must fit in {width} bits"));
    }
    let data = context.memory.read(start, len)?;
    let result = crc(&data, width, poly, init, xor_out, refin, refout);
    context.vars.insert(name, Value::Integer(result));
    Ok(())
}
