use std::fs;
use std::path::Path;

use crate::memory::Memory;
use crate::Result;

pub(crate) type Record = (u64, Vec<u8>);
type LoadedRecords = (Vec<Record>, Option<u64>);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Format {
    Bin,
    Ihex,
    Srec,
    S19,
    S28,
    S37,
    Thex,
}

impl Format {
    pub(crate) fn parse(s: &str) -> Result<Self> {
        match s.to_ascii_lowercase().as_str() {
            "bin" => Ok(Self::Bin),
            "ihex" => Ok(Self::Ihex),
            "srec" => Ok(Self::Srec),
            "s19" => Ok(Self::S19),
            "s28" => Ok(Self::S28),
            "s37" => Ok(Self::S37),
            "thex" => Ok(Self::Thex),
            _ => Err(format!("unsupported file format: {s}")),
        }
    }
}

pub(crate) fn read_file_as(
    path: &str,
    forced: Option<Format>,
) -> Result<(Format, Vec<Record>, Option<u64>)> {
    let data = fs::read(path).map_err(|e| format!("cannot read {path}: {e}"))?;
    let text = String::from_utf8_lossy(&data);
    let format = forced.unwrap_or_else(|| {
        let trimmed = text.trim_start();
        let extension = Path::new(path)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if extension == "bin" {
            Format::Bin
        } else if trimmed.starts_with(':') || extension == "hex" || extension == "ihex" {
            Format::Ihex
        } else if trimmed.starts_with('S')
            || extension == "srec"
            || extension == "s19"
            || extension == "mot"
        {
            Format::Srec
        } else if trimmed.starts_with('/')
            || trimmed.starts_with('%')
            || extension == "thex"
            || extension == "tek"
        {
            Format::Thex
        } else {
            Format::Bin
        }
    });
    match format {
        Format::Bin => Ok((format, vec![(0, data)], None)),
        Format::Ihex => load_ihex(&text).map(|(r, s)| (format, r, s)),
        Format::Srec | Format::S19 | Format::S28 | Format::S37 => {
            load_srec(&text).map(|(r, s)| (format, r, s))
        }
        Format::Thex => load_thex(&text).map(|r| (format, r, None)),
    }
}

pub(crate) fn save_file(
    path: &str,
    format: Format,
    memory: &Memory,
    blanks: u8,
    offset: u64,
    start: u64,
) -> Result<()> {
    let bytes = match format {
        Format::Bin => {
            let (&first, _) = memory
                .bytes
                .iter()
                .next()
                .ok_or("cannot write an empty memory space")?;
            let (&last, _) = memory
                .bytes
                .iter()
                .next_back()
                .ok_or("cannot write an empty memory space")?;
            let offset = if offset == 0 { first } else { offset };
            if offset < first || offset > last {
                return Err("binary output offset is outside the memory space".into());
            }
            (offset..=last)
                .map(|address| memory.bytes.get(&address).copied().unwrap_or(blanks))
                .collect()
        }
        Format::Ihex => save_ihex(memory)?.into_bytes(),
        Format::Srec => save_srec(memory, start, None)?.into_bytes(),
        Format::S19 => save_srec(memory, start, Some(2))?.into_bytes(),
        Format::S28 => save_srec(memory, start, Some(3))?.into_bytes(),
        Format::S37 => save_srec(memory, start, Some(4))?.into_bytes(),
        Format::Thex => save_thex(memory)?.into_bytes(),
    };
    fs::write(path, bytes).map_err(|e| format!("cannot write {path}: {e}"))
}

fn checksum_ihex(bytes: &[u8]) -> u8 {
    (0u8).wrapping_sub(bytes.iter().fold(0u8, |sum, b| sum.wrapping_add(*b)))
}

fn load_ihex(data: &str) -> Result<LoadedRecords> {
    let mut records = Vec::new();
    let mut base = 0u64;
    let mut start = None;
    let mut eof = false;
    for (line_no, line) in data.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if eof {
            return Err(format!("Intel HEX data after EOF at line {}", line_no + 1));
        }
        let hex = line
            .strip_prefix(':')
            .ok_or_else(|| format!("invalid Intel HEX line {}", line_no + 1))?;
        if hex.len() % 2 != 0 {
            return Err(format!(
                "invalid Intel HEX record length at line {}",
                line_no + 1
            ));
        }
        let raw = (0..hex.len())
            .step_by(2)
            .map(|i| {
                u8::from_str_radix(&hex[i..i + 2], 16)
                    .map_err(|_| format!("invalid Intel HEX byte at line {}", line_no + 1))
            })
            .collect::<Result<Vec<_>>>()?;
        if raw.len() < 5
            || raw[0] as usize + 5 != raw.len()
            || raw.iter().fold(0u8, |a, b| a.wrapping_add(*b)) != 0
        {
            return Err(format!(
                "invalid Intel HEX count/checksum at line {}",
                line_no + 1
            ));
        }
        let address = u16::from_be_bytes([raw[1], raw[2]]) as u64;
        let count = raw[0] as usize;
        match raw[3] {
            0 if count > 0 => records.push((base + address, raw[4..4 + count].to_vec())),
            0 => {}
            1 if count == 0 => eof = true,
            2 if count == 2 => base = u16::from_be_bytes([raw[4], raw[5]]) as u64 * 16,
            4 if count == 2 => base = u16::from_be_bytes([raw[4], raw[5]]) as u64 * 65536,
            3 if count == 4 => {
                let cs = u16::from_be_bytes([raw[4], raw[5]]) as u64;
                let ip = u16::from_be_bytes([raw[6], raw[7]]) as u64;
                start = Some(cs * 16 + ip);
            }
            5 if count == 4 => {
                start = Some(u32::from_be_bytes([raw[4], raw[5], raw[6], raw[7]]) as u64)
            }
            kind => {
                return Err(format!(
                    "unsupported Intel HEX record type {kind} at line {}",
                    line_no + 1
                ))
            }
        }
    }
    if !eof {
        return Err("Intel HEX file has no EOF record".into());
    }
    Ok((records, start))
}

fn load_srec(data: &str) -> Result<LoadedRecords> {
    let mut records = Vec::new();
    let mut start = None;
    for (line_no, line) in data.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let bytes = line
            .strip_prefix('S')
            .ok_or_else(|| format!("invalid S-record line {}", line_no + 1))?;
        if bytes.len() < 5 || bytes.len() % 2 == 0 {
            return Err(format!("invalid S-record line {}", line_no + 1));
        }
        let kind = bytes.as_bytes()[0] as char;
        let raw = (3..bytes.len())
            .step_by(2)
            .map(|i| {
                u8::from_str_radix(&bytes[i..i + 2], 16)
                    .map_err(|_| format!("invalid S-record byte at line {}", line_no + 1))
            })
            .collect::<Result<Vec<_>>>()?;
        let count = u8::from_str_radix(&bytes[1..3], 16)
            .map_err(|_| format!("invalid S-record count at line {}", line_no + 1))?;
        if raw.is_empty()
            || count as usize != raw.len()
            || raw.iter().fold(count, |s, b| s.wrapping_add(*b)) != 0xFF
        {
            return Err(format!(
                "invalid S-record count/checksum at line {}",
                line_no + 1
            ));
        }
        let address_len = match kind {
            '0' | '1' | '5' | '9' => 2,
            '2' | '6' | '8' => 3,
            '3' | '7' => 4,
            _ => return Err(format!("unsupported S-record type S{kind}")),
        };
        if raw.len() < address_len + 1 {
            return Err(format!("short S-record at line {}", line_no + 1));
        }
        let address = raw[..address_len]
            .iter()
            .fold(0u64, |a, b| (a << 8) | *b as u64);
        let payload = if raw.len() > address_len + 1 {
            &raw[address_len..raw.len() - 1]
        } else {
            &[]
        };
        match kind {
            '1' | '2' | '3' => {
                if !payload.is_empty() {
                    records.push((address, payload.to_vec()));
                }
            }
            '7' | '8' | '9' => start = Some(address),
            _ => {}
        }
    }
    Ok((records, start))
}

fn load_thex(data: &str) -> Result<Vec<Record>> {
    let mut records = Vec::new();
    let mut eof = false;
    for (line_no, line) in data.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if eof {
            return Err(format!(
                "Tektronix HEX data after EOF at line {}",
                line_no + 1
            ));
        }
        let body = line
            .strip_prefix('%')
            .ok_or_else(|| format!("invalid Tektronix HEX line {}", line_no + 1))?;
        if body.len() < 7 || !body.is_ascii() {
            return Err(format!("invalid Tektronix HEX line {}", line_no + 1));
        }
        let count = u8::from_str_radix(&body[0..2], 16)
            .map_err(|_| format!("invalid Tektronix HEX count at line {}", line_no + 1))?;
        if count as usize != body.len() {
            return Err(format!(
                "invalid Tektronix HEX count at line {}",
                line_no + 1
            ));
        }
        let tag = u8::from_str_radix(&body[2..3], 16)
            .map_err(|_| format!("invalid Tektronix HEX tag at line {}", line_no + 1))?;
        let expected = u8::from_str_radix(&body[3..5], 16)
            .map_err(|_| format!("invalid Tektronix HEX checksum at line {}", line_no + 1))?;
        let address_len = usize::from_str_radix(&body[5..6], 16).map_err(|_| {
            format!(
                "invalid Tektronix HEX address length at line {}",
                line_no + 1
            )
        })?;
        if !(1..=15).contains(&address_len) || count < 6 + address_len as u8 {
            return Err(format!(
                "invalid Tektronix HEX address length at line {}",
                line_no + 1
            ));
        }
        let address_end = 6 + address_len;
        if body.len() < address_end || !(body.len() - address_end).is_multiple_of(2) {
            return Err(format!(
                "invalid Tektronix HEX data at line {}",
                line_no + 1
            ));
        }
        let address = u64::from_str_radix(&body[6..address_end], 16)
            .map_err(|_| format!("invalid Tektronix HEX address at line {}", line_no + 1))?;
        let payload = (address_end..body.len())
            .step_by(2)
            .map(|i| {
                u8::from_str_radix(&body[i..i + 2], 16)
                    .map_err(|_| format!("invalid Tektronix HEX byte at line {}", line_no + 1))
            })
            .collect::<Result<Vec<_>>>()?;
        let actual = thex_checksum(count, tag, address_len as u8, address, &payload);
        if actual != expected {
            return Err(format!(
                "invalid Tektronix HEX checksum at line {}",
                line_no + 1
            ));
        }
        match tag {
            6 => {
                if payload.is_empty() {
                    return Err(format!(
                        "empty Tektronix HEX data record at line {}",
                        line_no + 1
                    ));
                }
                records.push((address, payload));
            }
            8 if payload.is_empty() => eof = true,
            _ => {
                return Err(format!(
                    "unsupported Tektronix HEX record tag {tag} at line {}",
                    line_no + 1
                ))
            }
        }
    }
    if !eof {
        return Err("Tektronix HEX file has no EOF record".into());
    }
    Ok(records)
}

fn thex_checksum(count: u8, tag: u8, address_len: u8, address: u64, data: &[u8]) -> u8 {
    // Tektronix HEX checksums add the nibbles of the count, address, and data, plus tag and width.
    let mut sum = (count >> 4) + (count & 0x0F) + tag + address_len;
    for nibble in 0..address_len {
        sum = sum.wrapping_add(((address >> (nibble as u32 * 4)) & 0x0F) as u8);
    }
    for byte in data {
        sum = sum.wrapping_add(byte >> 4).wrapping_add(byte & 0x0F);
    }
    sum
}

fn thex_record(tag: u8, address: u64, address_len: u8, data: &[u8]) -> String {
    let count = (6 + address_len as usize + data.len() * 2) as u8;
    let checksum = thex_checksum(count, tag, address_len, address, data);
    let address_hex = format!("{address:0width$X}", width = address_len as usize);
    let data_hex = data.iter().map(|b| format!("{b:02X}")).collect::<String>();
    format!("%{count:02X}{tag:X}{checksum:02X}{address_len:X}{address_hex}{data_hex}\n")
}

fn save_ihex(memory: &Memory) -> Result<String> {
    let mut out = String::new();
    if memory.bytes.keys().next_back().copied().unwrap_or(0) > u32::MAX as u64 {
        return Err("Intel HEX supports only 32-bit addresses".into());
    }
    let mut upper = None;
    let mut iter = memory.bytes.iter().peekable();
    while let Some((&address, _)) = iter.peek().copied() {
        let high = (address >> 16) as u16;
        if upper != Some(high) {
            // Emit even a zero upper word when switching back to the low 64 KiB.
            out.push_str(&ihex_record(0, 4, &high.to_be_bytes()));
            upper = Some(high);
        }
        let mut bytes = Vec::new();
        let base = address;
        while bytes.len() < 16 {
            match iter.peek().copied() {
                Some((&next, &byte))
                    if next == base + bytes.len() as u64 && (next >> 16) == high as u64 =>
                {
                    bytes.push(byte);
                    iter.next();
                }
                _ => break,
            }
        }
        out.push_str(&ihex_record(base as u16, 0, &bytes));
    }
    out.push_str(&ihex_record(0, 1, &[]));
    Ok(out)
}

fn ihex_record(address: u16, kind: u8, data: &[u8]) -> String {
    let mut raw = vec![data.len() as u8, (address >> 8) as u8, address as u8, kind];
    raw.extend_from_slice(data);
    raw.push(checksum_ihex(&raw));
    format!(
        ":{}\n",
        raw.iter().map(|b| format!("{b:02X}")).collect::<String>()
    )
}

fn save_srec(memory: &Memory, start: u64, forced_address_len: Option<usize>) -> Result<String> {
    let max = memory
        .bytes
        .keys()
        .next_back()
        .copied()
        .unwrap_or(0)
        .max(start);
    if max > u32::MAX as u64 {
        return Err("Motorola S-record supports only 32-bit addresses".into());
    }
    let (data_kind, address_len, term_kind) = match forced_address_len {
        Some(2) => ('1', 2, '9'),
        Some(3) => ('2', 3, '8'),
        Some(4) => ('3', 4, '7'),
        Some(_) => return Err("invalid Motorola S-record address size".into()),
        None if max <= 0xFFFF => ('1', 2, '9'),
        None if max <= 0xFF_FFFF => ('2', 3, '8'),
        None => ('3', 4, '7'),
    };
    if address_len == 2 && max > 0xFFFF {
        return Err("memory or start address exceeds S19 16-bit address range".into());
    }
    if address_len == 3 && max > 0xFF_FFFF {
        return Err("memory or start address exceeds S28 24-bit address range".into());
    }
    let mut out = srec_line('0', 0, 2, b"");
    let mut iter = memory.bytes.iter().peekable();
    while let Some((&address, _)) = iter.peek().copied() {
        let mut bytes = Vec::new();
        while bytes.len() < 32 {
            match iter.peek().copied() {
                Some((&next, &byte)) if next == address + bytes.len() as u64 => {
                    bytes.push(byte);
                    iter.next();
                }
                _ => break,
            }
        }
        out.push_str(&srec_line(data_kind, address, address_len, &bytes));
    }
    out.push_str(&srec_line(term_kind, start, address_len, &[]));
    Ok(out)
}

fn srec_line(kind: char, address: u64, address_len: usize, data: &[u8]) -> String {
    let mut raw = Vec::new();
    for i in (0..address_len).rev() {
        raw.push((address >> (i * 8)) as u8);
    }
    raw.extend_from_slice(data);
    let count = (raw.len() + 1) as u8;
    let checksum = !raw.iter().fold(count, |sum, b| sum.wrapping_add(*b));
    raw.push(checksum);
    format!(
        "S{kind}{count:02X}{}\n",
        raw.iter().map(|b| format!("{b:02X}")).collect::<String>()
    )
}

fn save_thex(memory: &Memory) -> Result<String> {
    let mut out = String::new();
    let address_len = memory.bytes.keys().next_back().copied().unwrap_or(0);
    let address_len = (64 - address_len.leading_zeros() as usize)
        .max(1)
        .div_ceil(4);
    if address_len > 15 {
        return Err("Tektronix HEX supports addresses up to 60 bits".into());
    }
    let address_len = address_len as u8;
    let mut iter = memory.bytes.iter().peekable();
    while let Some((&address, _)) = iter.peek().copied() {
        let mut data = Vec::new();
        let max_data = ((249 - address_len as usize) / 2).min(32);
        while data.len() < max_data {
            match iter.peek().copied() {
                Some((&next, &byte)) if next == address + data.len() as u64 => {
                    data.push(byte);
                    iter.next();
                }
                _ => break,
            }
        }
        out.push_str(&thex_record(6, address, address_len, &data));
    }
    out.push_str(&thex_record(8, 0, address_len, &[]));
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn intel_hex_round_trip_and_address_bounds() {
        let mut memory = Memory::default();
        memory
            .insert(0x1234, &[0xDE, 0xAD, 0xBE, 0xEF], false)
            .unwrap();
        memory.insert(0x1_0000, &[0x42], false).unwrap();
        memory.insert(0x20, &[0x17], false).unwrap();
        let text = save_ihex(&memory).unwrap();
        let (records, _) = load_ihex(&text).unwrap();
        let mut round_trip = Memory::default();
        for (address, data) in records {
            round_trip.insert(address, &data, false).unwrap();
        }
        assert_eq!(round_trip.bytes, memory.bytes);
        let mut too_high = Memory::default();
        too_high.insert(0x1_0000_0000, &[1], false).unwrap();
        assert!(save_ihex(&too_high).is_err());
    }

    #[test]
    fn s_record_round_trip_preserves_data_and_start_address() {
        let mut memory = Memory::default();
        memory
            .insert(0x1_2345, &[0xDE, 0xAD, 0xBE, 0xEF], false)
            .unwrap();
        let text = save_srec(&memory, 0x1_2345, None).unwrap();
        let (records, start) = load_srec(&text).unwrap();
        assert_eq!(start, Some(0x1_2345));
        assert_eq!(records, vec![(0x1_2345, vec![0xDE, 0xAD, 0xBE, 0xEF])]);
    }

    #[test]
    fn format_parser_supports_documented_s_record_styles() {
        assert_eq!(Format::parse("s19").unwrap(), Format::S19);
        assert_eq!(Format::parse("s28").unwrap(), Format::S28);
        assert_eq!(Format::parse("s37").unwrap(), Format::S37);
    }

    #[test]
    fn tektronix_hex_round_trip_and_checksum_validation() {
        let mut memory = Memory::default();
        memory
            .insert(0x12345, &[0xDE, 0xAD, 0xBE, 0xEF], false)
            .unwrap();
        let text = save_thex(&memory).unwrap();
        let records = load_thex(&text).unwrap();
        assert_eq!(records, vec![(0x12345, vec![0xDE, 0xAD, 0xBE, 0xEF])]);
        let corrupted = text.replacen("DE", "DF", 1);
        assert!(load_thex(&corrupted).is_err());
    }
}
