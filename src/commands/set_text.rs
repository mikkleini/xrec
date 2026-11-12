use crate::args::Arguments;
use crate::context::Context;
use crate::value::{text_bytes, text_value, variable_name, Value};
use crate::Result;

pub(crate) fn run(args: &mut Arguments<'_>, context: &mut Context) -> Result<()> {
    let name = args.required()?;
    variable_name(&name)?;
    let value = args.required()?;
    let format = if args
        .peek()
        .map(|s| {
            ["ascii", "utf8", "utf16"]
                .iter()
                .any(|f| s.eq_ignore_ascii_case(f))
        })
        .unwrap_or(false)
    {
        Some(args.required()?)
    } else {
        None
    };
    let (content, inherited_format) = text_value(&value, &context.vars)?;
    let format = format
        .or(inherited_format)
        .unwrap_or_else(|| "ascii".into());
    text_bytes(&content, &format, false)?;
    context.vars.insert(name, Value::Text { content, format });
    Ok(())
}
