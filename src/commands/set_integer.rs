use crate::args::Arguments;
use crate::context::Context;
use crate::value::{integer_argument, variable_name, Value};
use crate::Result;

pub(crate) fn run(args: &mut Arguments<'_>, context: &mut Context) -> Result<()> {
    let name = args.required()?;
    variable_name(&name)?;
    let value = integer_argument(&args.required()?, &context.vars)?;
    context.vars.insert(name, Value::Integer(value));
    Ok(())
}
