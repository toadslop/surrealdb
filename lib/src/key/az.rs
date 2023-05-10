use derive::Key;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Serialize, Deserialize, Key)]
pub struct Az<'a> {
	__: u8,
	_a: u8,
	pub ns: &'a str,
	_b: u8,
	pub db: &'a str,
	_c: u8,
	_d: u8,
	_e: u8,
	pub az: &'a str,
}

pub fn new<'a>(ns: &'a str, db: &'a str, tb: &'a str) -> Az<'a> {
	Az::new(ns, db, tb)
}

pub fn prefix(ns: &str, db: &str) -> Vec<u8> {
	let mut k = super::database::new(ns, db).encode().unwrap();
	k.extend_from_slice(&[0x21, 0x61, 0x7a, 0x00]);
	k
}

pub fn suffix(ns: &str, db: &str) -> Vec<u8> {
	let mut k = super::database::new(ns, db).encode().unwrap();
	k.extend_from_slice(&[0x21, 0x61, 0x7a, 0xff]);
	k
}

impl<'a> Az<'a> {
	pub fn new(ns: &'a str, db: &'a str, az: &'a str) -> Self {
		Self {
			__: 0x2f, // /
			_a: 0x2a, // *
			ns,
			_b: 0x2a, // *
			db,
			_c: 0x21, // !
			_d: 0x61, // a
			_e: 0x7a, // z
			az,
		}
	}
}

#[cfg(test)]
mod tests {
	#[test]
	fn key() {
		use super::*;
		#[rustfmt::skip]
            let val = Az::new(
            "ns",
            "db",
            "test",
        );
		let enc = Az::encode(&val).unwrap();
		let dec = Az::decode(&enc).unwrap();
		assert_eq!(val, dec);
	}
}