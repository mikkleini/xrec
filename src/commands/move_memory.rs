use crate::args::Arguments;
use crate::context::Context;
use crate::value::expression;
use crate::Result;

pub(crate) fn run(args: &mut Arguments<'_>, context: &mut Context) -> Result<()> {
    let start = expression(&args.required()?, &context.vars)?;
    let len = expression(&args.required()?, &context.vars)?;
    let dest = expression(&args.required()?, &context.vars)?;
    let overwrite = args.consume_flag(&["ow", "overwrite"]);
    context.memory.copy(start, len, dest, overwrite, true)
}
