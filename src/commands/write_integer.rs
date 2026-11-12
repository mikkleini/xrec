use crate::args::Arguments;
use crate::context::Context;
use crate::value::{expression, integer_argument, integer_fits, smallest_bits};
use crate::Result;

pub(crate) fn run(args: &mut Arguments<'_>, context: &mut Context) -> Result<()> {
    let address = expression(&args.required()?, &context.vars)?;
    let value_text = args.required()?;
    let value = integer_argument(&value_text, &context.vars)?;
    let bits_arg = args.required()?;
    let (bits, endian) = if let Some(suffix) = bits_arg.strip_prefix('u') {
        let bits = crate::value::parse_unsigned(suffix)?;
        let endian = if args
            .peek()
            .map(|s| s.eq_ignore_ascii_case("big") || s.eq_ignore_ascii_case("little"))
            .unwrap_or(false)
        {
            args.required()?
        } else {
            "little".into()
        };
        (bits, endian)
    } else {
        let bits = if bits_arg.eq_ignore_ascii_case("auto") {
            smallest_bits(
                value,
                value_text.starts_with('-') && !context.vars.contains_key(&value_text),
            )
        } else {
            expression(&bits_arg, &context.vars)?
        };
        (bits, args.required()?)
    };
    if bits == 0 || bits > 64 || bits % 8 != 0 {
        return Err("integer bits must be a multiple of 8 from 8 to 64".into());
    }
    if !integer_fits(value, bits) {
        return Err(format!("integer value does not fit in {bits} bits"));
    }
    if !endian.eq_ignore_ascii_case("big") && !endian.eq_ignore_ascii_case("little") {
        return Err("endianness must be big or little".into());
    }
    let overwrite = args.consume_flag(&["ow", "overwrite"]);
    let mut bytes = value.to_be_bytes()[8 - (bits / 8) as usize..].to_vec();
    if endian.eq_ignore_ascii_case("little") {
        bytes.reverse();
    }
    context.memory.insert(address, &bytes, overwrite)
}
