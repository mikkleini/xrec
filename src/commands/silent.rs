use crate::context::Context;
use crate::Result;

pub(crate) fn run(context: &mut Context) -> Result<()> {
    context.silent = true;
    Ok(())
}
