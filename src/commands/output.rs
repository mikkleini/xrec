use crate::args::Arguments;
use crate::context::Context;
use crate::formats::{save_file, Format};
use crate::value::expression;
use crate::Result;

pub(crate) fn run(args: &mut Arguments<'_>, context: &mut Context) -> Result<()> {
    let path = args.required()?;
    let format = Format::parse(&args.required()?)?;
    let (blanks, offset) = if format == Format::Bin {
        let blanks = if args
            .peek()
            .map(|s| !crate::args::command_name(s))
            .unwrap_or(false)
        {
            let value = expression(&args.required()?, &context.vars)?;
            if value > u8::MAX as u64 {
                return Err("binary blanks value must fit in u8".into());
            }
            value as u8
        } else {
            0
        };
        let offset = if args
            .peek()
            .map(|s| !crate::args::command_name(s))
            .unwrap_or(false)
        {
            expression(&args.required()?, &context.vars)?
        } else {
            0
        };
        (blanks, offset)
    } else {
        (0, 0)
    };
    save_file(
        &path,
        format,
        &context.memory,
        blanks,
        offset,
        context.start_address,
    )?;
    if !context.silent {
        println!("Wrote {path}");
    }
    Ok(())
}
