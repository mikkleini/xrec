use crate::args::Arguments;
use crate::context::Context;
use crate::value::expression;
use crate::Result;

pub(crate) fn run(args: &mut Arguments<'_>, context: &mut Context) -> Result<()> {
    let start = expression(&args.required()?, &context.vars)?;
    let len = expression(&args.required()?, &context.vars)?;
    context.memory.check_empty(start, len)
}
