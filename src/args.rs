use crate::Result;

pub(crate) struct Arguments<'a> {
    tokens: &'a [String],
    index: &'a mut usize,
}

impl<'a> Arguments<'a> {
    pub(crate) fn new(tokens: &'a [String], index: &'a mut usize) -> Self {
        Self { tokens, index }
    }

    pub(crate) fn has_more(&self) -> bool {
        *self.index < self.tokens.len()
    }

    pub(crate) fn command(&mut self) -> Result<String> {
        let value = self
            .tokens
            .get(*self.index)
            .ok_or_else(|| "expected command".to_string())?
            .clone();
        if !value.starts_with('-') || !command_name(&value) {
            return Err(format!("unknown command or unexpected argument: {value}"));
        }
        *self.index += 1;
        Ok(value)
    }

    pub(crate) fn required(&mut self) -> Result<String> {
        let value = self
            .tokens
            .get(*self.index)
            .ok_or_else(|| "missing command argument".to_string())?;
        if command_name(value) {
            return Err(format!("missing argument before {value}"));
        }
        *self.index += 1;
        Ok(value.clone())
    }

    pub(crate) fn peek(&self) -> Option<&str> {
        self.tokens.get(*self.index).map(String::as_str)
    }

    pub(crate) fn peek_next_is_command(&self) -> bool {
        self.tokens
            .get(*self.index + 1)
            .map(|s| command_name(s))
            .unwrap_or(true)
    }

    pub(crate) fn advance(&mut self) {
        *self.index += 1;
    }

    pub(crate) fn consume_flag(&mut self, names: &[&str]) -> bool {
        if self
            .peek()
            .map(|s| names.iter().any(|name| s.eq_ignore_ascii_case(name)))
            .unwrap_or(false)
        {
            self.advance();
            true
        } else {
            false
        }
    }
}

pub(crate) fn command_name(s: &str) -> bool {
    matches!(
        s,
        "-h" | "--help"
            | "-v"
            | "--version"
            | "-s"
            | "-i"
            | "-o"
            | "-sa"
            | "-ce"
            | "-cg"
            | "-f"
            | "-e"
            | "-tb"
            | "-te"
            | "-c"
            | "-m"
            | "-st"
            | "-si"
            | "-sv"
            | "-csum"
            | "-crc8"
            | "-crc16"
            | "-crc32"
            | "-sha2"
            | "-sha3"
            | "-wt"
            | "-wi"
            | "-wb"
            | "-wv"
            | "-pm"
            | "-pv"
    )
}
