use std::collections::BTreeMap;

use crate::Result;

#[derive(Default)]
pub(crate) struct Memory {
    pub(crate) bytes: BTreeMap<u64, u8>,
}

impl Memory {
    pub(crate) fn end(start: u64, len: u64) -> Result<u64> {
        if len == 0 {
            return Err("length must be greater than zero".into());
        }
        start
            .checked_add(len)
            .ok_or_else(|| "address range overflows u64".into())
    }

    pub(crate) fn insert(&mut self, start: u64, data: &[u8], overwrite: bool) -> Result<()> {
        let len = u64::try_from(data.len()).map_err(|_| "data is too large")?;
        let end = Self::end(start, len)?;
        if !overwrite && self.bytes.range(start..end).next().is_some() {
            return Err(format!("memory collision in range 0x{start:X}..0x{end:X}"));
        }
        for (i, byte) in data.iter().enumerate() {
            self.bytes.insert(start + i as u64, *byte);
        }
        Ok(())
    }

    pub(crate) fn insert_records(
        &mut self,
        records: &[(u64, Vec<u8>)],
        offset: u64,
        overwrite: bool,
    ) -> Result<()> {
        let mut occupied = BTreeMap::new();
        // Validate every record first so a collision never leaves a partially loaded file.
        for (start, data) in records {
            if data.is_empty() {
                continue;
            }
            let address = start
                .checked_add(offset)
                .ok_or("input address plus offset overflows u64")?;
            let end = Self::end(address, data.len() as u64)?;
            for existing in self.bytes.range(address..end).map(|(a, _)| *a) {
                if !overwrite {
                    return Err(format!("memory collision at 0x{existing:X}"));
                }
            }
            for (i, byte) in data.iter().enumerate() {
                let at = address + i as u64;
                if !overwrite && occupied.insert(at, *byte).is_some() {
                    return Err(format!("overlapping input records at 0x{at:X}"));
                }
            }
        }
        for (start, data) in records {
            if data.is_empty() {
                continue;
            }
            let address = start + offset;
            self.insert(address, data, overwrite)?;
        }
        Ok(())
    }

    pub(crate) fn read(&self, start: u64, len: u64) -> Result<Vec<u8>> {
        let end = Self::end(start, len)?;
        (start..end)
            .map(|address| {
                self.bytes
                    .get(&address)
                    .copied()
                    .ok_or_else(|| format!("memory gap at 0x{address:X}"))
            })
            .collect()
    }

    pub(crate) fn erase(&mut self, start: u64, len: u64) -> Result<()> {
        let end = Self::end(start, len)?;
        self.bytes
            .retain(|address, _| *address < start || *address >= end);
        Ok(())
    }

    pub(crate) fn copy(
        &mut self,
        start: u64,
        len: u64,
        dest: u64,
        overwrite: bool,
        moving: bool,
    ) -> Result<()> {
        let source_end = Self::end(start, len)?;
        let dest_end = Self::end(dest, len)?;
        // Half-open ranges overlap when each starts before the other ends.
        let overlaps = start < dest_end && dest < source_end;
        if !moving && overlaps {
            return Err("copy source and destination may not overlap".into());
        }
        if moving && overlaps && !overwrite {
            return Err("moving to an overlapping range requires overwrite".into());
        }
        let data = self.read(start, len)?;
        if moving
            && !overwrite
            && self
                .bytes
                .range(dest..dest_end)
                .any(|(a, _)| *a < start || *a >= source_end)
        {
            return Err("move destination is not empty (use overwrite)".into());
        }
        if moving {
            // Save the source bytes first, then clear it so overlapping moves remain safe.
            self.erase(start, len)?;
        }
        self.insert(dest, &data, overwrite)
    }

    pub(crate) fn fill(
        &mut self,
        start: u64,
        len: u64,
        vector: &[u8],
        overwrite: bool,
    ) -> Result<()> {
        if vector.is_empty() {
            return Err("fill vector must not be empty".into());
        }
        let end = Self::end(start, len)?;
        if !overwrite {
            for address in start..end {
                if self.bytes.contains_key(&address) {
                    continue;
                }
                // Keep the pattern anchored at the fill start, even when occupied bytes are skipped.
                self.bytes
                    .insert(address, vector[((address - start) as usize) % vector.len()]);
            }
        } else {
            for address in start..end {
                self.bytes
                    .insert(address, vector[((address - start) as usize) % vector.len()]);
            }
        }
        Ok(())
    }

    pub(crate) fn check_empty(&self, start: u64, len: u64) -> Result<()> {
        let end = Self::end(start, len)?;
        if let Some((&address, _)) = self.bytes.range(start..end).next() {
            return Err(format!("memory is not empty at 0x{address:X}"));
        }
        Ok(())
    }

    pub(crate) fn check_contiguous(&self, start: u64, len: u64) -> Result<()> {
        let end = Self::end(start, len)?;
        for address in start..end {
            if !self.bytes.contains_key(&address) {
                return Err(format!("memory gap at 0x{address:X}"));
            }
        }
        Ok(())
    }

    pub(crate) fn trim(
        &mut self,
        start: u64,
        len: u64,
        byte: u8,
        alignment: u64,
        from_end: bool,
    ) -> Result<()> {
        if alignment == 0 {
            return Err("alignment must be greater than zero".into());
        }
        let end = Self::end(start, len)?;
        let mut count = 0u64;
        if from_end {
            for current in (start..end).rev() {
                if self.bytes.get(&current) != Some(&byte) {
                    break;
                }
                count += 1;
            }
        } else {
            for current in start..end {
                if self.bytes.get(&current) != Some(&byte) {
                    break;
                }
                count += 1;
            }
        }
        // Leave a partial alignment group in place rather than shifting later data off alignment.
        count -= count % alignment;
        if count > 0 {
            if from_end {
                self.erase(end - count, count)?;
            } else {
                self.erase(start, count)?;
            }
        }
        Ok(())
    }
}
