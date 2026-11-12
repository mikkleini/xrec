use crate::args::Arguments;
use crate::context::Context;
use crate::value::{variable_name, vector, Value};
use crate::Result;

pub(crate) fn run(args: &mut Arguments<'_>, context: &mut Context) -> Result<()> {
    let name = args.required()?;
    variable_name(&name)?;
    let bytes = vector(&args.required()?, &context.vars)?;
    context.vars.insert(name, Value::Vector(bytes));
    Ok(())
}
