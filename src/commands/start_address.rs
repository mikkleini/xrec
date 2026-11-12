use crate::args::Arguments;
use crate::context::Context;
use crate::value::expression;
use crate::Result;

pub(crate) fn run(args: &mut Arguments<'_>, context: &mut Context) -> Result<()> {
    context.start_address = expression(&args.required()?, &context.vars)?;
    Ok(())
}
