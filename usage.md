# X-Rec usage

xrex command1 command2 ... commandN

Same command can be given multiple times. Number of commands is not limited.

## Commands

| Command       | Syntax                                                          | Example | Description |
|---------------|------------------------------------------------------------------|---------|-------------|
| Help          | `-h`                                                             | `xrec -h` | Print program usage instruction. |
| Version       | `-v`                                                             | `xrec -v` | Print program version. |
| Silent        | `-s`                                                             | `xrec -s` | Use silent mode and don't print commands verbose info. Still prints errors, help and version |
| Input file    | `-i <file> [offset=0] [overwrite]`                               | `xrec -i data.hex 0x1000` | Load binary file and add it to the memory space to given offset. Auto-detects file format. |
| Output file   | `-o <file> <file_format> [offset=0]`                             | `xrec -o out.hex ihex` | Save memory space into file in given format. Offset relevant for binary file saving only to skip empty memory space. |
| Start address | `-sa <addr>`                                                     | `xrec -sa 0x08000000` | Set executable start address. Relevant for Motorola S-record file format. |
| Check empty   | `-ce <addr> <len>`                                               | `xrec -ce 0x3000 0x400` | Check if memory area is completely empty. If not, cancel operation. |  
| Check no-gaps | `-cg <addr> <len>`                                               | `xrec -cg <addr> <len>` | Check if memory area has any gaps. If any gap found, cancel operation. |
| Fill          | `-f <addr> <len> <vector> [overwrite]`                           | `xrec -f 0x0 0x100 0xFF` | Fill given memory area with byte vector. Can used to fill with single byte or a pattern. If overwrite flag given fills whole area with vector, if not, then fills only empty space. |
| Erase         | `-e <addr> <len>`                                                | `xrec -e 0x100 0x50` | Erases given memory area. |
| Trim begin    | `-tb <addr> <len> <byte> [alignment=1]`                            | `xrec -tb 0x0 0x8000 0xFF` | Erases beginning of the memory area that contains given byte value up to first different byte value. Maintains alignment if alignment value greater than 1 given. |
| Trim end      | `-te <addr> <len> <byte> [alignment=1]`                            | `xrec -te 0x0 0x8000 0xFF` | Erases end of the memory area that contains given byte value up to the last different byte value. Maintains alignment if aligment value greater than 1 given. |
| Copy          | `-c <addr> <len> <dest> [overwrite]`                             | `xrec -c 0x0 0x100 0x2000` | Copies defined memory area to given destination address. Destination cannot overlap with source area. If overwriten flag give, overwrite any previous data, if not, the command will fail if area is not empty. |
| Move          | `-m <addr> <len> <dest> [overwrite]`                             | `xrec -m 0x0 0x100 0x2000` | Moves defined memory area to given destionation address. Destination can be within source area if overwrite flag is given. Overwrite flag will permit overwriting any previous data, if not give, the command will fail if area is not empty. |
| Set text      | `-st <var> "<text>" [text_format]`                               | `xrec -st NAME "Demo"`     | Set text value to given variable. |
| Set integer   | `-si <var> <int>`                                                | `xrec -si LOC 0xA000`      | Set integer value (maximum 64 bits) to given variable. |
| Set vector    | `-sv <var> <vector>"`                                            | `xrec -sv KEY AABBCCDD"`   | Set vector value (1-64 bytes) to given variable. |
| Checksum      | `-csum <addr> <len> <bits (8,16,32,64)> <var>`                   | `xrec -csum 0x0 0x1000 16 CS`                                           | Calculate simple checksum of defined memory area and set the value to variable. bits defined the value size. Sum can overflow. Any gap will fail the operation. |
| CRC-8         | `-crc8 <addr> <len> <poly> <init> <xor> [rin] [rout] <var>`      | `xrec -crc8 0x0 0x1000 0x2F 0xFF 0xFF 0xFFFF CS`                        | Calculate 8-bit CRC of defined memory area and set the value to variable. Any gap will fail the operation. poly, init and xor arguments shall be byte values. |
| CRC-16        | `-crc16 <addr> <len> <poly> <init> <xor> [rin] [rout] <var>`     | `xrec -crc16 0x0 0x80000 0x1021 0x0000 0x0000 rin rout CS`              | Calculate 16-bit CRC of defined memory area and set the value to variable. Any gap will fail the operation. init and xor arguments shall be unsigned 16-bit values. |
| CRC-32        | `-crc32 <addr> <len> <poly> <init> <xor> [rin] [rout] <var>`     | `xrec -crc32 0x0 0x10000 0x04C11DB7 0xFFFFFFFF 0xFFFFFFFF rif rout CS`  | Calculate 32-bit CRC of defined memory area and set the value to variable. Any gap will fail the operation. init and xor arguments shall be unsigned 32-bit values. |
| SHA-2         | `-sha2 <addr> <len> <bits (224,256,384,512)> <var>`              | `xrec -sha2 0x0 0x100000 256 HASH`  | Calculate SHA-2 hash of defined memory area and set it as a vector value to variable. |
| SHA-3         | `-sha3 <addr> <len> <bits (224,256,384,512)> <var>`              | `xrec -sha2 0x0 0x100000 256 HASH`  | Calculate SHA-3 hash of defined memory area and set it as a vector value to variable. |
| Write text    | `-wt <addr> <text>/<var> [text_format=ascii] [nullterm] [overwrite]` | `xrec -wt 0x1000 "Hello"`           | Write given text or text variable value to given memory address. text_format defines the format to use, nullterm defines if null character/word shall be appended. Overwrite flag permits overwriting existing data. |
| Write integer | `-wi <addr> <int>/<var> <bits (auto,8-64) <endianness> [overwrite]`    | `xrec -wi 0x2000 1234 u32`          | Writen given integer or integer variable value to given memory address. bits argument sets the size of the integer (shall be multiple of 8) or if auto is used, smallest possible bits is auto-detected. endianness sets the byte order for bits greater than 8. Overwrite flag permits overwriting existing data.  |
| Write vector  | `-wb <addr> <vector>/<var> [overwrite]`                                | `xrec -wb 0x2000 DEADBEEF`          | Write given bytes vector or vector variable value to given memory address. Overwrite flag permits overwriting existing data.  |



| Arguments   | Format                                                      | Example |
|-------------|-------------------------------------------------------------|---------|
| addr, dest, len  | Decimal or hexadecimal unsigned integer. Maximum 64-bits.   | 0x1000, 1000h, 65536 |
| bits, len        | Decimal or hexadecimal unsigned positive (>0) integer. Maximum 64-bits. "auto" allowed for automatic bits size detection. | 256, 0x8000 |
| int, offset      | Decimal or hexadecimal signed integer. Maximum 64-bits.   | 1024, +5, -0x400 |
| byte        | Byte value in decimal or hexadecimal format                | 126, 0x80 |
| endianness  | Integer endianness. Options: <br>big, little | little |
| file        | Relative or absolute file path                             | data.hex, c:\project\data.hex   |
| file_format | Binary file format. Options: <br/>bin (Binary), ihex (Intel Hex), srec (Motoral S-record), thex (Tektronix Hex) | ihex |
| nullterm    | Text null-termination activation with "nullterm" or "nt" flag. When not give, termination not added. | nt |
| text        | Text between quotation marks. Allowed characters depend on the text format | "Version" |
| text_format | Text format. Options: <br/>ascii, utf8, utf16  | ascii |
| poly, init, xor  | Hexadecimal unsigned integer. Maximum value depends on the command.            | 0x0, 0xFFFF, 0x20000 |
| overwrite   | Overwrite permission given with "overwrite" or "ow" flag. When not given, command fails if data collides. | ow, [blank] |
| var         | Name of the variable. Only ASCII characters, numbers and underscore allowed. Maximum 16 | X1, SIZE
| vector      | Bytes vector with size of 1-64 bytes in hexadecimal format. Notation not required.             | AA, DEADBEEF |

All memory writing operations can have optional overwrite permitting option "ow".