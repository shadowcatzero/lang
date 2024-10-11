use core::fmt;

pub struct Padder<'buf> {
    buf: &'buf mut (dyn fmt::Write + 'buf),
    on_newline: bool,
}

impl fmt::Write for Padder<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for s in s.split_inclusive('\n') {
            if self.on_newline {
                self.buf.write_str("    ")?;
            }

            self.on_newline = s.ends_with('\n');
            self.buf.write_str(s)?;
        }

        Ok(())
    }

    fn write_char(&mut self, c: char) -> fmt::Result {
        if self.on_newline {
            self.buf.write_str("    ")?;
        }
        self.on_newline = c == '\n';
        self.buf.write_char(c)
    }
}

impl<'buf> Padder<'buf> {
    pub fn new(buf: &'buf mut (dyn fmt::Write + 'buf)) -> Self {
        Self {
            buf,
            on_newline: false,
        }
    }
}
