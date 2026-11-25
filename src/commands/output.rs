use crate::args::Arguments;
use crate::context::Context;
use crate::formats::{save_file, Format};
use crate::value::expression;
use crate::Result;

fn has_argument(args: &Arguments<'_>) -> bool {
    args.peek()
        .map(|s| !crate::args::command_name(s))
        .unwrap_or(false)
}

fn is_overwrite_flag(args: &Arguments<'_>) -> bool {
    args.peek()
        .map(|s| s.eq_ignore_ascii_case("ow") || s.eq_ignore_ascii_case("overwrite"))
        .unwrap_or(false)
}

pub(crate) fn run(args: &mut Arguments<'_>, context: &mut Context) -> Result<()> {
    let path = args.required()?;
    let format = Format::parse(&args.required()?)?;
    let address = if has_argument(args) && !is_overwrite_flag(args) {
        let value = args.required()?;
        if value == "-" {
            0
        } else {
            expression(&value, &context.vars)?
        }
    } else {
        0
    };
    let len = if has_argument(args) && !is_overwrite_flag(args) {
        let value = args.required()?;
        if value == "-" {
            None
        } else {
            Some(expression(&value, &context.vars)?)
        }
    } else {
        None
    };
    let overwrite = args.consume_flag(&["ow", "overwrite"]);
    if args.peek() == Some("-") {
        args.advance();
    }
    save_file(
        &path,
        format,
        &context.memory,
        address,
        len,
        context.start_address,
        overwrite,
    )?;
    if !context.silent {
        println!("Wrote {path}");
    }
    Ok(())
}
