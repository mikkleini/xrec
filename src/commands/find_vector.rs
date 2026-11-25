use crate::args::Arguments;
use crate::context::Context;
use crate::value::{expression, variable_name, vector, Value};
use crate::Result;

pub(crate) fn run(args: &mut Arguments<'_>, context: &mut Context) -> Result<()> {
    let start = expression(&args.required()?, &context.vars)?;
    let len = expression(&args.required()?, &context.vars)?;
    let direction = expression(&args.required()?, &context.vars)?;
    let pattern = vector(&args.required()?, &context.vars)?;
    let name = args.required()?;
    variable_name(&name)?;
    if context.vars.contains_key(&name) {
        return Err(format!("variable already exists: {name}"));
    }
    if direction != 1 && direction != u64::MAX {
        return Err("find direction must be 1 or -1".into());
    }
    let end = crate::memory::Memory::end(start, len)?;
    let pattern_len = u64::try_from(pattern.len()).map_err(|_| "vector is too large to search")?;
    let last_start = end
        .checked_sub(pattern_len)
        .filter(|last| *last >= start)
        .ok_or("vector is longer than the search area")?;

    let candidates = || {
        context
            .memory
            .bytes
            .range(start..end)
            .filter_map(|(&address, &byte)| {
                (address <= last_start && byte == pattern[0]).then_some(address)
            })
    };
    let found = if direction == 1 {
        candidates().find(|address| matches_at(&context.memory.bytes, *address, &pattern))
    } else {
        candidates()
            .rev()
            .find(|address| matches_at(&context.memory.bytes, *address, &pattern))
    }
    .ok_or("vector not found in search area")?;

    context.vars.insert(name, Value::Integer(found));
    Ok(())
}

fn matches_at(memory: &std::collections::BTreeMap<u64, u8>, address: u64, pattern: &[u8]) -> bool {
    pattern
        .iter()
        .enumerate()
        .all(|(i, byte)| address.checked_add(i as u64).and_then(|at| memory.get(&at)) == Some(byte))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::args::Arguments;

    fn find(context: &mut Context, tokens: &[&str]) -> Result<()> {
        let tokens = tokens.iter().map(|s| s.to_string()).collect::<Vec<_>>();
        let mut index = 0;
        run(&mut Arguments::new(&tokens, &mut index), context)
    }

    #[test]
    fn finds_first_match_in_each_direction() {
        let mut context = Context::default();
        context.memory.insert(0x100, &[0xAA, 0xBB], false).unwrap();
        context.memory.insert(0x103, &[0xAA, 0xBB], false).unwrap();

        find(&mut context, &["0x100", "5", "1", "[AA BB]", "FORWARD"]).unwrap();
        find(&mut context, &["0x100", "5", "-1", "[AA BB]", "REVERSE"]).unwrap();

        assert_eq!(context.vars.get("FORWARD"), Some(&Value::Integer(0x100)));
        assert_eq!(context.vars.get("REVERSE"), Some(&Value::Integer(0x103)));
    }

    #[test]
    fn does_not_match_across_gaps_or_overwrite_existing_variables() {
        let mut context = Context::default();
        context.memory.insert(0x100, &[0xAA], false).unwrap();
        context.memory.insert(0x102, &[0xBB], false).unwrap();
        assert!(find(&mut context, &["0x100", "3", "1", "[AA BB]", "FOUND"]).is_err());
        context.vars.insert("FOUND".into(), Value::Integer(7));
        assert!(find(&mut context, &["0x100", "3", "1", "[AA]", "FOUND"]).is_err());
    }
}
