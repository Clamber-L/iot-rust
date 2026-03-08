use std::path::Path;
use serde::Deserialize;

use crate::error::{IotError, IotResult};
use crate::protocol::field::{Encoding, FieldDef, FieldValue, ParsedFrame};

/// TOML-loadable protocol definition.
#[derive(Debug, Deserialize)]
pub struct ProtocolDef {
    pub fields: Vec<FieldDef>,
}

impl ProtocolDef {
    pub fn from_file(path: impl AsRef<Path>) -> IotResult<Self> {
        let content = std::fs::read_to_string(path)?;
        let def: ProtocolDef = toml::from_str(&content)?;
        Ok(def)
    }
}

pub struct FrameParser {
    fields: Vec<FieldDef>,
}

impl FrameParser {
    pub fn new(fields: Vec<FieldDef>) -> Self {
        Self { fields }
    }

    pub fn from_protocol_def(def: ProtocolDef) -> Self {
        Self::new(def.fields)
    }

    pub fn parse(&self, data: &[u8]) -> IotResult<ParsedFrame> {
        let mut frame = ParsedFrame::default();
        let mut cursor = 0usize;

        for field_def in &self.fields {
            let length = if let Some(fixed) = field_def.length {
                fixed
            } else if let Some(ref src) = field_def.length_from {
                frame
                    .get(src)
                    .and_then(|v| v.as_usize())
                    .ok_or_else(|| {
                        IotError::FrameError(format!(
                            "length_from field '{}' not found or not an integer",
                            src
                        ))
                    })?
            } else {
                return Err(IotError::FrameError(format!(
                    "field '{}' has neither 'length' nor 'length_from'",
                    field_def.name
                )));
            };

            let end = cursor + length;
            if end > data.len() {
                return Err(IotError::FrameError(format!(
                    "field '{}': need {} bytes at offset {}, but data is only {} bytes",
                    field_def.name,
                    length,
                    cursor,
                    data.len()
                )));
            }

            let bytes = &data[cursor..end];
            let value = decode_field(bytes, &field_def.encoding)?;
            frame.push(&field_def.name, value);
            cursor = end;
        }

        Ok(frame)
    }
}

fn decode_field(bytes: &[u8], encoding: &Encoding) -> IotResult<FieldValue> {
    match encoding {
        Encoding::U8 => {
            require_len(bytes, 1, "U8")?;
            Ok(FieldValue::U8(bytes[0]))
        }
        Encoding::I8 => {
            require_len(bytes, 1, "I8")?;
            Ok(FieldValue::I8(bytes[0] as i8))
        }
        Encoding::BigEndianU16 => {
            require_len(bytes, 2, "BigEndianU16")?;
            Ok(FieldValue::U16(u16::from_be_bytes([bytes[0], bytes[1]])))
        }
        Encoding::LittleEndianU16 => {
            require_len(bytes, 2, "LittleEndianU16")?;
            Ok(FieldValue::U16(u16::from_le_bytes([bytes[0], bytes[1]])))
        }
        Encoding::BigEndianI16 => {
            require_len(bytes, 2, "BigEndianI16")?;
            Ok(FieldValue::I16(i16::from_be_bytes([bytes[0], bytes[1]])))
        }
        Encoding::LittleEndianI16 => {
            require_len(bytes, 2, "LittleEndianI16")?;
            Ok(FieldValue::I16(i16::from_le_bytes([bytes[0], bytes[1]])))
        }
        Encoding::BigEndianU32 => {
            require_len(bytes, 4, "BigEndianU32")?;
            Ok(FieldValue::U32(u32::from_be_bytes(
                bytes[..4].try_into().unwrap(),
            )))
        }
        Encoding::LittleEndianU32 => {
            require_len(bytes, 4, "LittleEndianU32")?;
            Ok(FieldValue::U32(u32::from_le_bytes(
                bytes[..4].try_into().unwrap(),
            )))
        }
        Encoding::BigEndianI32 => {
            require_len(bytes, 4, "BigEndianI32")?;
            Ok(FieldValue::I32(i32::from_be_bytes(
                bytes[..4].try_into().unwrap(),
            )))
        }
        Encoding::LittleEndianI32 => {
            require_len(bytes, 4, "LittleEndianI32")?;
            Ok(FieldValue::I32(i32::from_le_bytes(
                bytes[..4].try_into().unwrap(),
            )))
        }
        Encoding::BigEndianU64 => {
            require_len(bytes, 8, "BigEndianU64")?;
            Ok(FieldValue::U64(u64::from_be_bytes(
                bytes[..8].try_into().unwrap(),
            )))
        }
        Encoding::LittleEndianU64 => {
            require_len(bytes, 8, "LittleEndianU64")?;
            Ok(FieldValue::U64(u64::from_le_bytes(
                bytes[..8].try_into().unwrap(),
            )))
        }
        Encoding::Gbk => {
            let (cow, _, had_errors) = encoding_rs::GBK.decode(bytes);
            if had_errors {
                return Err(IotError::FrameError(
                    "GBK decode encountered unmappable bytes".into(),
                ));
            }
            Ok(FieldValue::Text(cow.into_owned()))
        }
        Encoding::Utf8 => {
            let s = std::str::from_utf8(bytes).map_err(|e| {
                IotError::FrameError(format!("UTF-8 decode error: {}", e))
            })?;
            Ok(FieldValue::Text(s.to_owned()))
        }
        Encoding::Bcd => {
            let mut s = String::with_capacity(bytes.len() * 2);
            for &b in bytes {
                s.push(char::from_digit((b >> 4) as u32, 10).unwrap_or('?'));
                s.push(char::from_digit((b & 0x0f) as u32, 10).unwrap_or('?'));
            }
            Ok(FieldValue::Text(s))
        }
        Encoding::Bytes => Ok(FieldValue::Bytes(bytes.to_vec())),
    }
}

fn require_len(bytes: &[u8], expected: usize, enc: &str) -> IotResult<()> {
    if bytes.len() != expected {
        Err(IotError::FrameError(format!(
            "{} requires exactly {} bytes, got {}",
            enc,
            expected,
            bytes.len()
        )))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Example GB26875 frame (from protocol docs, hex-encoded).
    /// @@ device_id(2) cmd(1) len(1) addr(2) ddl(2) time_tag(7) cs(1) ##
    /// Build a minimal synthetic frame for the unit test:
    ///   start:              40 40
    ///   device_id:          67 68
    ///   command:            01
    ///   length:             00
    ///   address:            00 00
    ///   data_domain_length: 00 07
    ///   time_tag:           26 03 08 12 00 00 00  (BCD: 260308120000 00)
    ///   checksum:           XOR of bytes[2..cs_pos]
    ///   end:                23 23
    #[test]
    fn test_parse_gb26875_frame() {
        // Build raw bytes first, then compute checksum
        let mut frame: Vec<u8> = vec![
            0x40, 0x40, // start @@
            0x67, 0x68, // device_id
            0x01,       // command
            0x00,       // length
            0x00, 0x00, // address
            0x00, 0x07, // data_domain_length
            0x26, 0x03, 0x08, 0x12, 0x00, 0x00, 0x00, // time_tag (7 bytes BCD)
        ];
        let cs: u8 = frame[2..].iter().fold(0u8, |acc, &b| acc ^ b);
        frame.push(cs);          // checksum
        frame.push(0x23);        // end ##
        frame.push(0x23);

        let def = ProtocolDef::from_file("protocols/gb26875.toml").unwrap();
        let parsed = FrameParser::from_protocol_def(def).parse(&frame).unwrap();

        assert_eq!(parsed.get("command"), Some(&FieldValue::U8(0x01)));
    }
}
