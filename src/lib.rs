mod args;
mod checksum;
mod commands;
mod context;
mod formats;
mod memory;
mod value;

use args::Arguments;
use context::Context;

pub type Result<T> = std::result::Result<T, String>;

pub fn run(tokens: &[String]) -> Result<()> {
    let mut context = Context::default();
    let mut index = 0;
    let mut args = Arguments::new(tokens, &mut index);
    let mut did_command = false;
    while args.has_more() {
        let command = args.command()?;
        commands::dispatch(&command, &mut args, &mut context)?;
        did_command = true;
    }
    if !did_command {
        commands::show_help()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::checksum::crc;
    use crate::memory::Memory;
    use crate::value::{
        encode_text, expression, integer_argument, integer_fits, parse_unsigned, smallest_bits,
        vector, Value,
    };
    use std::collections::HashMap;

    #[test]
    fn memory_insert_is_atomic_on_collision() {
        let mut memory = Memory::default();
        memory.insert(10, &[1, 2], false).unwrap();
        assert!(memory.insert(9, &[3, 4, 5], false).is_err());
        assert_eq!(memory.read(10, 2).unwrap(), [1, 2]);
    }

    #[test]
    fn parses_hex_and_expression_variables() {
        let mut vars = HashMap::new();
        vars.insert("BASE".into(), Value::Integer(0x1000));
        assert_eq!(parse_unsigned("1000h").unwrap(), 0x1000);
        assert_eq!(expression("BASE+16", &vars).unwrap(), 0x1010);
        assert_eq!(expression("BASE-0x10", &vars).unwrap(), 0xFF0);
        assert_eq!(integer_argument("BASE+16", &vars).unwrap(), 0x1010);
        assert_eq!(integer_argument("-5", &vars).unwrap(), u64::MAX - 4);
    }

    #[test]
    fn crc_standard_check_value() {
        assert_eq!(crc(b"123456789", 16, 0x1021, 0xFFFF, 0, true, true), 0x6F91);
    }

    #[test]
    fn vectors_accept_each_documented_hex_notation() {
        let vars = HashMap::new();
        assert_eq!(
            vector("[0x11,0x22,0x33]", &vars).unwrap(),
            [0x11, 0x22, 0x33]
        );
        assert_eq!(vector("DEADBEEF", &vars).unwrap(), [0xDE, 0xAD, 0xBE, 0xEF]);
    }

    #[test]
    fn fill_preserves_existing_data_without_overwrite() {
        let mut memory = Memory::default();
        memory.insert(5, &[0xAA], false).unwrap();
        memory.fill(4, 3, &[0xFF], false).unwrap();
        assert_eq!(memory.read(4, 3).unwrap(), [0xFF, 0xAA, 0xFF]);
    }

    #[test]
    fn move_overlapping_source_preserves_moved_bytes() {
        let mut memory = Memory::default();
        memory
            .insert(0x100, &[0xAA, 0xBB, 0xCC, 0xDD], false)
            .unwrap();
        memory.copy(0x100, 4, 0x102, true, true).unwrap();
        assert_eq!(memory.read(0x102, 4).unwrap(), [0xAA, 0xBB, 0xCC, 0xDD]);
        assert!(memory.read(0x100, 2).is_err());
    }

    #[test]
    fn input_records_are_inserted_atomically_with_offset() {
        let mut memory = Memory::default();
        memory.insert(0x20, &[0xFF], false).unwrap();
        let records = vec![(0, vec![1, 2]), (0x20, vec![3])];
        assert!(memory.insert_records(&records, 0, false).is_err());
        assert!(memory.read(0, 2).is_err());
        assert_eq!(memory.read(0x20, 1).unwrap(), [0xFF]);

        memory
            .insert_records(&[(0, vec![1, 2])], 0x100, false)
            .unwrap();
        assert_eq!(memory.read(0x100, 2).unwrap(), [1, 2]);
    }

    #[test]
    fn automatic_integer_width_handles_signed_values() {
        assert_eq!(smallest_bits((-1i64) as u64, true), 8);
        assert_eq!(smallest_bits((-129i64) as u64, true), 16);
        assert_eq!(smallest_bits(0x100, false), 16);
        assert!(integer_fits((-128i64) as u64, 8));
        assert!(!integer_fits((-129i64) as u64, 8));
        assert!(!integer_fits(256, 8));
    }

    #[test]
    fn text_variable_preserves_its_encoding_when_written() {
        let mut vars = HashMap::new();
        vars.insert(
            "LABEL".into(),
            Value::Text {
                content: "A".into(),
                format: "utf16".into(),
            },
        );
        assert_eq!(
            encode_text("LABEL", None, false, &vars).unwrap(),
            [0x41, 0x00]
        );
        assert_eq!(
            encode_text("LABEL", Some("utf8"), false, &vars).unwrap(),
            [0x41]
        );
    }

    #[test]
    fn sequential_commands_write_variables_and_memory() {
        let args = [
            "-si", "ADDR", "0x100", "-st", "TXT", "OK", "-wt", "ADDR", "TXT", "-pm",
        ]
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<_>>();
        assert!(run(&args).is_ok());
    }
}
