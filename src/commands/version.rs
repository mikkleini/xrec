use crate::Result;

pub(crate) fn run() -> Result<()> {
    println!("xrec {}", env!("CARGO_PKG_VERSION"));
    Ok(())
}
