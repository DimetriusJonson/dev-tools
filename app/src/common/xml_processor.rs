use std::io::Cursor;

use quick_xml::{
    Reader, Writer,
    events::{BytesText, Event},
};
use quick_xml::escape::{escape, unescape};

pub fn format_xml(xml: &str, ident: usize) -> Result<String, Box<dyn std::error::Error>> {
    let mut input_xml_reader = Reader::from_str(xml);
    input_xml_reader.config_mut().trim_text(true);

    let mut read_buffer = Vec::new();
    let mut writer = Writer::new_with_indent(Cursor::new(&mut read_buffer), b' ', ident);

    loop {
        match input_xml_reader.read_event()? {
            Event::Text(ref e) => {
                let text_content = input_xml_reader.decoder().decode(e)?;
                let filtered_lines: Vec<&str> =
                    text_content.lines().filter(|line| !line.trim().is_empty()).collect();

                if !filtered_lines.is_empty() {
                    let filtered_text = filtered_lines.join("\n");
                    writer.write_event(Event::Text(BytesText::new(&filtered_text)))?;
                }
            }
            Event::Comment(e) => writer.write_event(Event::Comment(e))?,
            Event::Eof => break,
            event => writer.write_event(event)?,
        };
    }

    Ok(String::from_utf8(read_buffer)?)
}

pub fn escape_xml(xml: &str, escape_xml: bool) -> Result<String, Box<dyn std::error::Error>> {
    let mut input_xml_reader = Reader::from_str(xml);
    input_xml_reader.config_mut().trim_text(true);

    let mut read_buffer = Vec::new();
    let mut writer = Writer::new(Cursor::new(&mut read_buffer));

    loop {
        match input_xml_reader.read_event()? {
            Event::Eof => break,
            event => writer.write_event(event)?,
        };
    }

    let str = String::from_utf8(read_buffer)?;
    let converted_str = match escape_xml {
        true => escape(str),
        false => unescape(&str)?,
    };

    Ok(converted_str.to_string())
}
