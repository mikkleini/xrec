use crate::args::{command_name, Arguments};
use crate::context::Context;
use crate::formats::{read_file_as, Format};
use crate::value::expression;
use crate::Result;

pub(crate) fn run(args: &mut Arguments<'_>, context: &mut Context) -> Result<()> {
    let path = args.required()?;
    let forced = if args
        .peek()
        .map(|s| {
            s == "-"
                || matches!(
                    s.to_ascii_lowercase().as_str(),
                    "auto" | "bin" | "ihex" | "srec" | "s19" | "s28" | "s37" | "thex"
                )
        })
        .unwrap_or(false)
    {
        let type_name = args.required()?.to_ascii_lowercase();
        if type_name == "auto" || type_name == "-" {
            None
        } else {
            Some(Format::parse(&type_name)?)
        }
    } else {
        None
    };
    let offset = if args
        .peek()
        .map(|s| {
            !command_name(s) && !["ow", "overwrite"].contains(&s.to_ascii_lowercase().as_str())
        })
        .unwrap_or(false)
    {
        let value = args.required()?;
        if value == "-" {
            0
        } else {
            expression(&value, &context.vars)?
        }
    } else {
        0
    };
    let overwrite = args.consume_flag(&["ow", "overwrite"]);
    let (_, records, file_start) = read_file_as(&path, forced)?;
    context.memory.insert_records(&records, offset, overwrite)?;
    if let Some(address) = file_start {
        context.start_address = address
            .checked_add(offset)
            .ok_or("start address plus input offset overflows u64")?;
    }
    if !context.silent {
        println!("Loaded {path}");
    }
    Ok(())
}
