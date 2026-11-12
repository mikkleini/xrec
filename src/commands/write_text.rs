use crate::args::Arguments;
use crate::context::Context;
use crate::value::{encode_text, expression};
use crate::Result;

pub(crate) fn run(args: &mut Arguments<'_>, context: &mut Context) -> Result<()> {
    let address = expression(&args.required()?, &context.vars)?;
    let value = args.required()?;
    let mut format = None;
    let mut nullterm = false;
    let mut overwrite = false;
    while let Some(option) = args.peek() {
        if ["ascii", "utf8", "utf16"]
            .iter()
            .any(|candidate| option.eq_ignore_ascii_case(candidate))
        {
            format = Some(args.required()?);
        } else if args.consume_flag(&["nt", "nullterm"]) {
            nullterm = true;
        } else if args.consume_flag(&["ow", "overwrite"]) {
            overwrite = true;
        } else {
            break;
        }
    }
    let bytes = encode_text(value.as_str(), format.as_deref(), nullterm, &context.vars)?;
    context.memory.insert(address, &bytes, overwrite)
}
