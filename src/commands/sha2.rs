use crate::args::Arguments;
use crate::context::Context;
use crate::Result;

pub(crate) fn run(args: &mut Arguments<'_>, context: &mut Context) -> Result<()> {
    super::sha::run(args, context, false)
}
