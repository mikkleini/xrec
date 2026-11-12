use crate::args::Arguments;
use crate::context::Context;
use crate::value::{expression, vector};
use crate::Result;

pub(crate) fn run(args: &mut Arguments<'_>, context: &mut Context) -> Result<()> {
    let start = expression(&args.required()?, &context.vars)?;
    let len = expression(&args.required()?, &context.vars)?;
    let pattern = vector(&args.required()?, &context.vars)?;
    let overwrite = args.consume_flag(&["ow", "overwrite"]);
    context.memory.fill(start, len, &pattern, overwrite)
}
