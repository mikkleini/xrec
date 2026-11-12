use crate::context::Context;
use crate::value::Value;
use crate::Result;

pub(crate) fn run(context: &Context) -> Result<()> {
    let mut names: Vec<_> = context.vars.keys().collect();
    names.sort();
    for name in names {
        let value = match &context.vars[name] {
            Value::Integer(value) => format!("0x{value:X} ({value})"),
            Value::Text { content, format } => format!("{content:?} ({format})"),
            Value::Vector(bytes) => bytes.iter().map(|b| format!("{b:02X}")).collect::<String>(),
        };
        println!("{name} = {value}");
    }
    Ok(())
}
