# Xrec

Xrec is a cross-platform command line utility to create, convert and modify binary, Intel Hex, Tektronix Hex and Motorola S-record files. Its primary use case is embedded firmware development. It can perform multiple operations in single execution. For example, it can merge multiple files, relocate them to the dedicated addresses, write headers and footers, calculate checksum and then save it all down into single or multiple files.

Xrec is written in Rust largely by AI tools, because I never had time to write it on my own. So I rather wrote complete usage specification and let the AI do the dirty work.

# Xrec usage

`xrec -command1 arg1 arg2 -command2 arg1 arg2 arg3 ... -commandN arg1`

Each command has own arguments. Same command can be given multiple times (probably with different arguments).
Number of commands is not limited, but OS specific call length still applies.

xrec keeps the memory between commands and allows each command to alter that memory.
Similarly it keeps the variables.
Technically it is possible to process binary file(s) with multiple xrec calls by exporting memory into temporary file(s) and importing it/them at next call.

Arguments convention:
  * <> is mandatory argument
  * [] is optional argument with possible default value after =

If optional argument is given, all preceding optional arguments shall be given.
  
## Supported file types

Import and export commands support following file types:

* Binary (extensions: .bin, .raw).
* Intel Hex (extensions: .hex, .mcs, .int, .ihex, .ihe, .ihx).
* Motorola S-record (extensions: .s19, .s28, .s37, .s, .s1, .s2, .s3, .sx, .srec, .mot, .mxt).
* Tektronix Hex (extension: .tek).

## Commands quick reference

| Command                         | Syntax                                                         |
| ------------------------------- | -------------------------------------------------------------- |
| [Help](#help)                   | `-h`                                                           |
| [Version](#version)             | `-v`                                                           |
| [Silent](#silent)               | `-s`                                                           |
| [Import file](#import-file)     | `-i <file> [type=auto] [offset=0] [overwrite]`                 |
| [Export file](#export-file)     | `-o <file> <type> [blanks=0] [offset=0]`                       |
| [Start address](#start-address) | `-sa <addr>`                                                   |
| [Check empty](#check-empty)     | `-ce <addr> <len>`                                             |
| [Check gaps](#check-gaps)       | `-cg <addr> <len>`                                             |
| [Fill](#fill)                   | `-f <addr> <len> <vector> [overwrite]`                         |
| [Erase](#erase)                 | `-e <addr> <len>`                                              |
| [Trim begin](#trim-begin)       | `-tb <addr> <len> <byte> [align=1]`                            |
| [Trim end](#trim-end)           | `-te <addr> <len> <byte> [align=1]`                            |
| [Copy](#copy)                   | `-c <addr> <len> <dest> [overwrite]`                           |
| [Move](#move)                   | `-m <addr> <len> <dest> [overwrite]`                           |
| [Set text](#set-text)           | `-st <var> <text> [format=ascii]`                              |
| [Set integer](#set-integer)     | `-si <var> <int>`                                              |
| [Set vector](#set-vector)       | `-sv <var> <vector>`                                           |
| [Checksum](#checksum)           | `-csum <addr> <len> <bits> <var>`                              |
| [CRC-8](#crc-8)                 | `-crc8 <addr> <len> <poly> <init> <xor> [rin] [rout] <var>`    |
| [CRC-16](#crc-16)               | `-crc16 <addr> <len> <poly> <init> <xor> [rin] [rout] <var>`   |
| [CRC-32](#crc-32)               | `-crc32 <addr> <len> <poly> <init> <xor> [rin] [rout] <var>`   |
| [SHA-2](#sha-2)                 | `-sha2 <addr> <len> <bits> <var>`                              |
| [SHA-3](#sha-3)                 | `-sha3 <addr> <len> <bits> <var>`                              |
| [Write text](#write-text)       | `-wt <addr> <text> [format=ascii] [nullterm] [overwrite]`      |
| [Write integer](#write-integer) | `-wi <addr> <int> <bits> <endianness> [overwrite]`             |
| [Write vector](#write-vector)   | `-wb <addr> <vector> [overwrite]`                              |
| [Print memory](#print-memory)   | `-pm`                                                          |
| [Print vars](#print-vars)       | `-pv`                                                          |

## Commands in details

### Help

`-h`

Print instructions.

### Version

`-v`

Print program version number.

### Silent

`-s`

Turn off program verbose output. Still prints errors, help and version.
  
### Import file

`-i <file> [type=auto] [offset=0] [overwrite]`

Import file into memory. Any number of files can be imported, but their overlap shall be prevented unless `overwrite` flag is given.
Arguments:

  * *file* is relative or absolute path of the imported file. File type is automatically detected. Supported types are:
  * *type* is one of the following file types:
    * *auto* auto-detects file type by extension. This is default option when argument is not given.
    * *bin* enforces parsing binary file.
    * *ihex* enforces parsing Intel Hex.
    * *srec* enforces parsing Motorola S-record.
    * *thex* enforces parsing Tektronix Hex.
  * *offset* is optional address offset to add to imported file memory content. Offset data type is signed [integer](#integer).
  * *overwrite* or *ow* flag can be set to overwrite any existing memory space. If overwrite flag is not set and imported file content overlaps with existing memory content, then error is given.

### Export file

`-o <file> <type> [blanks=0] [offset=0]`                          

Exports memory into file. Memory content remains intact after export. Memory can be exported in multiple formats.
Arguments:

  * *file* is relative or absolute path of the exported file.
  * *type* is one of the following file types:
    * *bin* to save as binary file.
    * *ihex* to save as Intel Hex.
    * *srec* to save as Motorola S-record (smallest possible address style).
    * *s19* to save as Motorola S-record with 16-bit addresses.
    * *s28* to save as Motorola S-record with 24-bit addresses.
    * *s37* to save as Motorola S-record with 32-bit addresses.
    * *thex* to save as Tektronix Hex.
  * *blanks* is optional argument for binary file type used to fill memory gaps. Blanks date type is [integer](#integer) in 0-255 range. Usually it is either 0x00 or 0xFF.
  * *offset* is optional argument for binary file type to start saving from given offset. E.g. when data starts at 0x100000 then it is possible to skip saving 1MB of blank data by giving 0x100000 offset. Offset data type is unsigned [integer](#integer).

### Start address

`-sa <addr>`

Set executable start address. Relevant when exporting Motorola S-record file type.
Arguments:
  * *addr* is execution address as unsigned [integer](#integer).

### Check empty

`-ce <addr> <len>`

Check if memory area is completely empty. If not empty, program will stop and return error.
Arguments:
  * *addr* is start address of the area as unsigned [integer](#integer).
  * *len* is length of the area in bytes as positive [integer](#integer).

### Check gaps

`-cg <addr> <len>`

Check if memory area has any gaps. If any gap found, program wil stop and returns error.
Arguments:
  * *addr* is start address of the area as unsigned [integer](#integer).
  * *len* is length of the area in bytes as positive [integer](#integer).

### Fill

`-f <addr> <len> <vector> [overwrite]`

Fill given memory area with byte vector. Can used to fill with single byte or a pattern. Pattern is aligned to start address and vector length even if single byte gaps are filled.
Arguments:
  * *addr* is start address of the area as unsigned [integer](#integer).
  * *len* is length of the area in bytes as positive [integer](#integer).
  * *vector* is fill pattern as byte(s) [vector](#vector).
  * *overwrite* or *ow* flag can be set to overwrite any existing memory. If overwrite flag is not given, fills only the blank spaces.

### Erase

`-e <addr> <len>`

Erases defined memory area.
Arguments:
  * *addr* is start address of the area as unsigned [integer](#integer).
  * *len* is length of the area in bytes as positive [integer](#integer).

### Trim begin

`-tb <addr> <len> <byte> [align=1]`

Erases beginning of the memory area that contains given byte value up to the first different byte value. Maintains alignment if alignment value greater than 1 is given.
Arguments:
  * *addr* is start address of the area as unsigned [integer](#integer).
  * *len* is length of the area in bytes as positive [integer](#integer).
  * *byte* is the byte value to trim away as [integer](#integer) in 0-255 range.
  * *align* is optional alignment to maintain while trimming. Alignment is given as positive [integer](#integer).

Example:

Assume the memory area has following 16 bytes:

`FF FF FF FF FF FF FF FF
FF FF FF FF DE AD BE EF`

If 0xFF is trimmed with alignment 1, then result is:

`.. .. .. .. .. .. .. ..
.. .. .. .. DE AD BE EF`

.. marks the empty pace.

If 0xFF is trimmed with alignment 8, then result is:

`.. .. .. .. .. .. .. ..
FF FF FF FF DE AD BE EF`

### Trim end

`-te <addr> <len> <byte> [align=1]`

Similar to [trim begin](#trim-begin), erases end of the memory area that contains given byte value up to the last different byte value. Maintains alignment if alignment value greater than 1 is given.
Arguments:
  * *addr* is start address of the area as unsigned [integer](#integer).
  * *len* is length of the area in bytes as positive [integer](#integer).
  * *byte* is the byte value to trim away as [integer](#integer) in 0-255 range.
  * *align* is optional alignment to maintain while trimming. Alignment is given as positive [integer](#integer).

### Copy

`-c <addr> <len> <dest> [overwrite]`

Copies defined memory area to given destination address. Destination cannot overlap with source area.
Arguments:
  * *addr* is start address of the source area as unsigned [integer](#integer).
  * *len* is length of the area in bytes as positive [integer](#integer).
  * *dest* is destination start address as unsigned [integer](#integer).
  * *overwrite* or *ow* flag can be set to overwrite any existing memory at the destination. If overwrite flag is not set and the destination content overlaps with existing memory content, then error is given.

### Move

`-m <addr> <len> <dest> [overwrite]`

Moves defined memory area to given destination address. Destination can be within source area.
Arguments:
  * *addr* is start address of the source area as unsigned [integer](#integer).
  * *len* is length of the area in bytes as positive [integer](#integer).
  * *dest* is destination start address as unsigned [integer](#integer).
  * *overwrite* or *ow* flag can be set to overwrite any existing memory at the destination. If overwrite flag is not set and the destination content overlaps with existing memory content outside of the moved area, then error is given.

### Set text

`-st <var> <text> [format]`

Set text value to given variable.
Arguments:
  * *var* is name of the variable to set.
  * *text* is [text](#text).

### Set integer

`-si <var> <int>`

Set integer value to given variable.
Arguments:
  * *var* is name of the variable to set.
  * *int* is [integer](#integer).

### Set vector

`-sv <var> <vector>`

Set vector value to given variable.
Arguments:
  * *var* is name of the variable to set.
  * *vector* is [vector](#vector).

### Checksum

`-csum <addr> <len> <bits> <var>`

Calculate simple checksum of defined memory area and set the value to variable. bits defines the value size. Sum can overflow (wraps around). Any gap in the memory area will fail the operation.
Arguments:
  * *addr* is start address of the area as unsigned [integer](#integer).
  * *len* is length of the area in bytes as positive [integer](#integer).
  * *bits* is the result size in bits: 8, 16, 32 or 64.
  * *var* is name of the variable the result is stored into. If variable already exists, command will fail.

### CRC-8

`-crc8 <addr> <len> <poly> <init> <xor> [rin] [rout] <var>`

Calculate 8-bit CRC of defined memory area and set the value to variable. Any gap in the memory area will fail the operation.
Arguments:
  * *addr* is start address of the area as unsigned [integer](#integer).
  * *len* is length of the area in bytes as positive [integer](#integer).
  * *poly*, *init* and *xor* shall be byte values as unsigned [integer](#integer) in 0 to 255 range.
  * *rin* optional flag reflects each input byte before it is processed.
  * *rout* optional flag reflects the final result before the xor value is applied.
  * *var* is name of the variable the result is stored into. If variable already exists, command will fail.

### CRC-16

`-crc16 <addr> <len> <poly> <init> <xor> [rin] [rout] <var>`

Calculate 16-bit CRC of defined memory area and set the value to variable. Any gap in the memory area will fail the operation.
Arguments:
  * *addr* is start address of the area as unsigned [integer](#integer).
  * *len* is length of the area in bytes as positive [integer](#integer).
  * *poly*, *init* and *xor* shall be unsigned 16-bit values as unsigned (hexa)decimal integer or expression.
  * *rin* optional flag reflects each input byte before it is processed.
  * *rout* optional flag reflects the final result before the xor value is applied.
  * *var* is name of the variable the result is stored into. If variable already exists, command will fail.

### CRC-32

`-crc32 <addr> <len> <poly> <init> <xor> [rin] [rout] <var>`

Calculate 32-bit CRC of defined memory area and set the value to variable. Any gap in the memory area will fail the operation.
Arguments:
  * *addr* is start address of the area as unsigned [integer](#integer).
  * *len* is length of the area in bytes as positive [integer](#integer).
  * *poly*, *init* and *xor* shall be unsigned 32-bit values as unsigned (hexa)decimal integer or expression.
  * *rin* optional flag reflects each input byte before it is processed.
  * *rout* optional flag reflects the final result before the xor value is applied.
  * *var* is name of the variable the result is stored into. If variable already exists, command will fail.

### SHA-2

`-sha2 <addr> <len> <bits> <var>`

Calculate SHA-2 hash of defined memory area and set it as a vector value to variable.
Arguments:
  * *addr* is start address of the area as unsigned [integer](#integer).
  * *len* is length of the area in bytes as positive [integer](#integer).
  * *bits* is the hash size in bits: 224, 256, 384 or 512.
  * *var* is name of the variable the resulting vector is stored into.

### SHA-3

`-sha3 <addr> <len> <bits> <var>`

Calculate SHA-3 hash of defined memory area and set it as a vector value to variable.
Arguments:
  * *addr* is start address of the area as unsigned [integer](#integer).
  * *len* is length of the area in bytes as positive [integer](#integer).
  * *bits* is the hash size in bits: 224, 256, 384 or 512.
  * *var* is name of the variable the resulting vector is stored into.

### Write text

`-wt <addr> <text> [format=ascii] [nullterm] [overwrite]`

Write given text or text variable value to given memory address.
Arguments:
  * *addr* is memory address to write to, as unsigned [integer](#integer).
  * *text* is [text](#text).
  * *format* is optional text format to use: ascii, utf8 or utf16. Default is ascii.
  * *nullterm* or *nt* flag appends a null character (or null word, depending on format) after the text.
  * *overwrite* or *ow* flag can be set to overwrite any existing memory content. If overwrite flag is not set and the write overlaps with existing memory content, then error is given.

### Write integer

`-wi <addr> <int> <bits> [endianness] [overwrite]`

Write given integer to given memory address. If value does not fit into specified bits, error is given.
Arguments:
  * *addr* is memory address to write to, as unsigned [integer](#integer).
  * *int* is [integer](#integer).
  * *bits* sets the size of the integer. Options:
    * *auto* to auto-detect the sign and smallest possible size (8-bit aligned),
    * *u8* or *i8* for 8-bit (un)signed representation.
    * *u16* or *i16* for 16-bit (un)signed representation.
    * *u24* or *i24* for 24-bit (un)signed representation.
    * *u32* or *i32* for 32-bit (un)signed representation.
    * *u64* or *i64* for 64-bit (un)signed representation.
  * *endianness* sets the byte order: *big* or *little*. Appies for bits size greater than 8.
  * *overwrite* or *ow* flag can be set to overwrite any existing memory content. If overwrite flag is not set and the write overlaps with existing memory content, then error is given.

### Write vector

`-wv <addr> <vector> [overwrite]`

Write given bytes vector or vector variable value to given memory address.
Arguments:
  * *addr* is memory address to write to, as unsigned [integer](#integer).
  * *vector* is [vector](#vector).
  * *overwrite* or *ow* flag can be set to overwrite any existing memory content. If overwrite flag is not set and the write overlaps with existing memory content, then error is given.

### Print memory

`-pm`

Print out all memory blocks - their start address, length, first 4 bytes (in hex) and last 4 bytes (in hex).

### Print vars

`-pv`

Print out all variables and their values.

## Argument types

### Integer

All integers are signed 64-bit numbers inside xrec. Commands may restrict the value to be positive and limit the range of the value.
Integers can be written in decimal and hexadecimal format. Hexadecimal format can be used with "0x" prefix or "h" suffix.
Integer arguments can be given as expressions made of integer and/or variable names and + or - operators. Expressions shall not contain spaces.

Example expression:

`HDR_OFFSET+0x10000-16h+3`

### Text

Text argument shall be given in printable ACII or UTF-8 format with surrounding quotation marks (").
Quotation mark can be used within text with escape character \\.
Line break is given with \n and carrier return with \r.

Example:

`"Name: \"Bob\""`

### Vector

Vector argument contains one or more bytes (up to 64). Bytes are given between \[ and \] brackets in hexadecimal (not decimal!) form without notation. Separators , : ; - and space are optional and if separator is used, it shall be consistent.

Examples:

`[DEADBEEF]
[DE-AD-BE-EF]
[DE AD BE EF]`

## Build and install

Install the Rust toolchain, then build the executable with `cargo build --release`.
The resulting binary is available at `target/release/xrec` (or `xrec.exe` on Windows).
Run the test suite with `cargo test`.

# Xrec design

Xrec is written in Rust. Each command is implemented in its own module under `src/commands`,
which makes Xrec extendable with custom commands (requires re-compiling). Shared memory operations,
argument and value parsing, checksums, and binary file-format support live in separate modules
and are reused by the command modules.

Xrec has a "memory space" which is single or multiple non-overlapping memory blocks on which the operations are performed.
It also has variable dictionary which holds variables and their values between commands.
Integer values are all stored as i64 and converted if command requires different type.
