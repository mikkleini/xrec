use crate::args::Arguments;
use crate::context::Context;
use crate::value::{expression, variable_name, Value};
use crate::Result;

pub(crate) fn run(args: &mut Arguments<'_>, context: &mut Context) -> Result<()> {
    let start = expression(&args.required()?, &context.vars)?;
    let len = expression(&args.required()?, &context.vars)?;
    let bits = expression(&args.required()?, &context.vars)?;
    if ![8, 16, 32, 64].contains(&bits) {
        return Err("checksum bits must be 8, 16, 32, or 64".into());
    }
    let name = args.required()?;
    variable_name(&name)?;
    if context.vars.contains_key(&name) {
        return Err(format!("variable already exists: {name}"));
    }
    let data = context.memory.read(start, len)?;
    let mask = if bits == 64 {
        u64::MAX
    } else {
        (1u64 << bits) - 1
    };
    let sum = data
        .iter()
        .fold(0u64, |sum, byte| sum.wrapping_add(*byte as u64))
        & mask;
    context.vars.insert(name, Value::Integer(sum));
    Ok(())
}
