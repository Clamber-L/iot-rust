use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum Encoding {
    U8,
    I8,
    BigEndianU16,
    LittleEndianU16,
    BigEndianI16,
    LittleEndianI16,
    BigEndianU32,
    LittleEndianU32,
    BigEndianI32,
    LittleEndianI32,
    BigEndianU64,
    LittleEndianU64,
    Gbk,
    Utf8,
    Bcd,
    Bytes,
}

#[derive(Debug, Clone, Deserialize)]
pub struct FieldDef {
    pub name: String,
    /// Fixed length in bytes. Mutually exclusive with `length_from`.
    pub length: Option<usize>,
    /// Dynamic length: name of a previously parsed field whose value gives the byte count.
    pub length_from: Option<String>,
    pub encoding: Encoding,
}

#[derive(Debug, Clone)]
pub enum FieldValue {
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    I8(i8),
    I16(i16),
    I32(i32),
    Text(String),
    Bytes(Vec<u8>),
}

impl FieldValue {
    /// Convert integer variants to `usize`, used for `length_from` resolution.
    pub fn as_usize(&self) -> Option<usize> {
        match self {
            FieldValue::U8(v) => Some(*v as usize),
            FieldValue::U16(v) => Some(*v as usize),
            FieldValue::U32(v) => Some(*v as usize),
            FieldValue::U64(v) => Some(*v as usize),
            FieldValue::I8(v) if *v >= 0 => Some(*v as usize),
            FieldValue::I16(v) if *v >= 0 => Some(*v as usize),
            FieldValue::I32(v) if *v >= 0 => Some(*v as usize),
            _ => None,
        }
    }
}

impl PartialEq for FieldValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (FieldValue::U8(a), FieldValue::U8(b)) => a == b,
            (FieldValue::U16(a), FieldValue::U16(b)) => a == b,
            (FieldValue::U32(a), FieldValue::U32(b)) => a == b,
            (FieldValue::U64(a), FieldValue::U64(b)) => a == b,
            (FieldValue::I8(a), FieldValue::I8(b)) => a == b,
            (FieldValue::I16(a), FieldValue::I16(b)) => a == b,
            (FieldValue::I32(a), FieldValue::I32(b)) => a == b,
            (FieldValue::Text(a), FieldValue::Text(b)) => a == b,
            (FieldValue::Bytes(a), FieldValue::Bytes(b)) => a == b,
            _ => false,
        }
    }
}

/// Ordered collection of named parsed values for a single frame.
#[derive(Debug, Default)]
pub struct ParsedFrame {
    fields: Vec<(String, FieldValue)>,
}

impl ParsedFrame {
    pub fn push(&mut self, name: impl Into<String>, value: FieldValue) {
        self.fields.push((name.into(), value));
    }

    pub fn get(&self, name: &str) -> Option<&FieldValue> {
        self.fields.iter().find(|(n, _)| n == name).map(|(_, v)| v)
    }

    pub fn fields(&self) -> &[(String, FieldValue)] {
        &self.fields
    }
}
