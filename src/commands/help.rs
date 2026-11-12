use crate::Result;

pub(crate) fn run() -> Result<()> {
    println!(
        "xrec -command [arguments] ...\n\
         Commands: -h, -v, -s, -i, -o, -sa, -ce, -cg, -f, -e, -tb, -te, -c, -m,\n\
         -st, -si, -sv, -csum, -crc8, -crc16, -crc32, -sha2, -sha3, -wt, -wi, -wb,\n\
         -pm, -pv. Use -i FILE [offset] [ow] and -o FILE bin|ihex|srec|thex [offset].\n\
         Numeric arguments accept decimal, 0x-prefixed hex, and trailing-h hex."
    );
    Ok(())
}
