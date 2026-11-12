use crate::context::Context;
use crate::Result;

pub(crate) fn run(context: &Context) -> Result<()> {
    let mut iter = context.memory.bytes.iter().peekable();
    while let Some((&start, _)) = iter.peek().copied() {
        let mut data = Vec::new();
        let mut address = start;
        // Consume one contiguous run; the first address after a gap starts the next block.
        while let Some((&next, &byte)) = iter.peek().copied() {
            if next != address {
                break;
            }
            data.push(byte);
            iter.next();
            address += 1;
        }
        let first = data
            .iter()
            .take(4)
            .map(|b| format!("{b:02X}"))
            .collect::<String>();
        // Reverse the tail back into address order for display.
        let last = data
            .iter()
            .rev()
            .take(4)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .map(|b| format!("{b:02X}"))
            .collect::<String>();
        println!(
            "0x{start:016X} len={} first={first} last={last}",
            data.len()
        );
    }
    Ok(())
}
