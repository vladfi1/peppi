use std::io::{Read, Result, Error, ErrorKind};
use std::collections::HashMap;

use byteorder::{BigEndian, ReadBytesExt};

#[derive(Debug, PartialEq, Eq, Clone, serde::Serialize)]
#[serde(untagged)]
pub enum Object {
	Int(i64),
	Map(HashMap<String, Object>),
	Str(String),
}

query_impl!(Object, self, f, config, query {
	match self {
		Object::Int(i) => i.query(f, config, query),
		Object::Str(s) => s.query(f, config, query),
		Object::Map(m) => m.query(f, config, query),
	}
});

pub trait ToObject {
	fn to_object(self) -> Object;
}

impl ToObject for i64 {
	fn to_object(self) -> Object {
		Object::Int(self)
	}
}

impl ToObject for String {
	fn to_object(self) -> Object {
		Object::Str(self)
	}
}

impl ToObject for &str {
	fn to_object(self) -> Object {
		Object::Str(self.to_string())
	}
}

impl ToObject for HashMap<String, Object> {
	fn to_object(self) -> Object {
		Object::Map(self)
	}
}

fn parse_utf8<R:Read>(r:&mut R) -> Result<String> {
	let length = r.read_u8()?;
	let mut buf = vec![0; length as usize];
	r.read_exact(&mut buf)?;
	String::from_utf8(buf).map_err(|e| Error::new(ErrorKind::InvalidData, e))
}

fn parse_val<R:Read>(r:&mut R) -> Result<Object> {
	match r.read_u8()? {
		0x53 => { // "S": str
			match r.read_u8()? {
				0x55 => Ok(Object::Str(parse_utf8(r)?)),
				c => Err(Error::new(ErrorKind::InvalidData, format!("Expected 0x55 for string length, but got: {}", c))),
			}
		},
		0x6c => { // "l": i32
			Ok(Object::Int(r.read_i32::<BigEndian>()? as i64))
		},
		0x7b => { // "{": map
			Ok(Object::Map(parse_map(r)?))
		}
		c => Err(Error::new(ErrorKind::InvalidData, format!("unexpected UBJSON value type: {}", c)))
	}
}

fn parse_key<R:Read>(r:&mut R) -> Result<Option<String>> {
	match r.read_u8()? {
		0x55 => Ok(Some(parse_utf8(r)?)),
		0x7d => Ok(None),
		c => Err(Error::new(ErrorKind::InvalidData, format!("unexpected UBJSON key type: {}", c)))
	}
}

pub fn parse_map<R:Read>(r:&mut R) -> Result<HashMap<String, Object>> {
	let mut m = HashMap::new();
	while match parse_key(r)? {
		Some(k) => {m.insert(k, parse_val(r)?); true},
		None => false,
	} {}
	Ok(m)
}
