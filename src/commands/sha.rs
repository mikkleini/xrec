use crate::args::Arguments;
use crate::checksum::hash;
use crate::context::Context;
use crate::value::{expression, variable_name, Value};
use crate::Result;

pub(crate) fn run(args: &mut Arguments<'_>, context: &mut Context, sha3: bool) -> Result<()> {
    let start = expression(&args.required()?, &context.vars)?;
    let len = expression(&args.required()?, &context.vars)?;
    let bits = expression(&args.required()?, &context.vars)?;
    let name = args.required()?;
    variable_name(&name)?;
    let data = context.memory.read(start, len)?;
    context
        .vars
        .insert(name, Value::Vector(hash(&data, bits, sha3)?));
    Ok(())
}
