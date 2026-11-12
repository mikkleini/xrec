use crate::args::Arguments;
use crate::context::Context;
use crate::value::expression;
use crate::Result;

pub(crate) fn run(args: &mut Arguments<'_>, context: &mut Context, from_end: bool) -> Result<()> {
    let start = expression(&args.required()?, &context.vars)?;
    let len = expression(&args.required()?, &context.vars)?;
    let byte = expression(&args.required()?, &context.vars)?;
    if byte > u8::MAX as u64 {
        return Err("trim byte must fit in u8".into());
    }
    let alignment = if args
        .peek()
        .map(|s| !crate::args::command_name(s) && expression(s, &context.vars).is_ok())
        .unwrap_or(false)
    {
        expression(&args.required()?, &context.vars)?
    } else {
        1
    };
    context
        .memory
        .trim(start, len, byte as u8, alignment, from_end)
}
