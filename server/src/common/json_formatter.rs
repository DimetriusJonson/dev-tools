use bytes::Bytes;

pub struct JsonFormatter {
    ident: usize,
    escaped: bool,
    in_string: bool,
    indent_level: usize,
    newline_requested: bool, // invalidated if next character is ] or }
}

impl JsonFormatter {
    pub fn new(ident: usize) -> Self {
        Self { ident, escaped: false, in_string: false, indent_level: 0, newline_requested: false }
    }

    pub fn format_bytes(&mut self, data: Bytes) -> Bytes {
        let mut formatted_bytes = Vec::<u8>::with_capacity(1024);
        for char in data {
            if self.in_string {
                let mut escape_here = false;
                match char {
                    b'"' if !self.escaped => {
                        self.in_string = false;
                    }
                    b'\\' if !self.escaped => {
                        escape_here = true;
                    }
                    _ => {}
                }
                formatted_bytes.push(char);
                self.escaped = escape_here;
            } else {
                let mut auto_push = true;
                let mut request_newline = false;
                let old_level = self.indent_level;

                match char {
                    b'"' => self.in_string = true,
                    b' ' | b'\n' | b'\r' | b'\t' => continue,
                    b'[' => {
                        self.indent_level += 1;
                        request_newline = true;
                    }
                    b'{' => {
                        self.indent_level += 1;
                        request_newline = true;
                    }
                    b'}' | b']' => {
                        self.indent_level = self.indent_level.saturating_sub(1);
                        if !self.newline_requested {
                            // see comment below about newline_requested
                            formatted_bytes.push(b'\n');
                            Self::write_ident(&mut formatted_bytes, self.indent_level, self.ident);
                        }
                    }
                    b':' => {
                        auto_push = false;
                        formatted_bytes.push(char);
                        formatted_bytes.push(b' ');
                    }
                    b',' => {
                        request_newline = true;
                    }
                    _ => {}
                }
                if self.newline_requested && char != b']' && char != b'}' {
                    // newline only happens after { [ and ,
                    // this means we can safely assume that it being followed up by } or ]
                    // means an empty object/array
                    formatted_bytes.push(b'\n');
                    Self::write_ident(&mut formatted_bytes, old_level, self.ident);
                }

                if auto_push {
                    formatted_bytes.push(char);
                }

                self.newline_requested = request_newline;
            }
        }

        // trailing newline
        formatted_bytes.push(b'\n');

        let chunk = Bytes::copy_from_slice(&formatted_bytes);
        formatted_bytes.clear();

        chunk
    }

    fn write_ident(write_buffer: &mut Vec<u8>, level: usize, ident: usize) -> () {
        for _ in 0..level * ident {
            write_buffer.push(b' ');
        }
    }
}
