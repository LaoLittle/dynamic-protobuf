pub mod macros;

use std::collections::HashMap;
use std::default::Default;

use bytes::{Bytes};

#[derive(Default)]
pub struct DynamicMessage {
    inner: HashMap<u64, DynamicMessageNode>,
}

impl DynamicMessage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn encode(self) -> Bytes {
        let mut encoder = DynamicMessageEncoder::new();

        self._encode(&mut encoder);

        encoder.into_bytes()
    }

    pub fn encode_to_vec(self) -> Vec<u8> {
        let mut encoder = DynamicMessageEncoder::new();

        self._encode(&mut encoder);

        encoder.into_vec()
    }

    fn _encode(self,encoder: &mut DynamicMessageEncoder) {
        for (id, msg) in self.inner {
            let key = id << 3;

            match msg {
                DynamicMessageNode::I64(val) => {
                    encoder.put_uvarint(key);
                    encoder.put_svarint(val)
                }
                DynamicMessageNode::U64(val) => {
                    encoder.put_uvarint(key);
                    encoder.put_uvarint(val);
                }
                DynamicMessageNode::F32(val) => {
                    encoder.put_uvarint(key | 5);
                    encoder.put_u32_bits(val.to_bits())
                }
                DynamicMessageNode::F64(val) => {
                    encoder.put_uvarint(key | 1);
                    encoder.put_u64_bits(val.to_bits());
                }
                DynamicMessageNode::RepeatU64(vec) => {
                    for val in vec {
                        encoder.put_uvarint(key);
                        encoder.put_uvarint(val);
                    }
                }
                DynamicMessageNode::Bytes(b) => {
                    encoder.put_uvarint(key | 2);
                    encoder.put_uvarint(b.len() as _);
                    encoder.buf.extend(b);
                }
            }
        }
    }

    pub fn set<M: Into<DynamicMessageNode>>(&mut self, k: u64, v: M) {
        self.inner.insert(k, v.into());
    }
}

struct DynamicMessageEncoder {
    buf: Vec<u8>,
}

impl DynamicMessageEncoder {
    //const MAX_VARINT_LEN16: usize = 3;
    //const MAX_VARINT_LEN32: usize = 5;
    const MAX_VARINT_LEN64: usize = 10;

    pub fn new() -> Self {
        Self {
            buf: Vec::new(),
        }
    }

    pub fn put_uvarint(&mut self, v: u64) {
        let mut b = [0; Self::MAX_VARINT_LEN64];
        let n = _put_uvarint(&mut b, v);
        self.buf.extend(&b[..n]);
    }

    pub fn put_svarint(&mut self, v: i64) {
        self.put_uvarint((v as u64) << 1 ^ ((v >> 63) as u64))
    }

    pub fn put_u32_bits(&mut self, v: u32) {
        self.buf.extend(v.to_le_bytes())
    }

    pub fn put_u64_bits(&mut self, v: u64) {
        self.buf.extend(v.to_le_bytes())
    }

    pub fn into_bytes(self) -> Bytes {
        Bytes::from(self.buf)
    }

    pub fn into_vec(self) -> Vec<u8> {
        self.buf
    }
}

fn _put_uvarint(buf: &mut [u8], mut x: u64) -> usize {
    let mut i = 0;
    while x >= 0x80 {
        buf[i] = (x as u8) | 0x80;
        x >>= 7;
        i += 1;
    }
    buf[i] = x as u8;
    i + 1
}

pub enum DynamicMessageNode {
    I64(i64),
    U64(u64),
    F32(f32),
    F64(f64),
    Bytes(Bytes),
    RepeatU64(Vec<u64>),
}

impl From<bool> for DynamicMessageNode {
    fn from(val: bool) -> Self {
        Self::U64(if val { 1 } else { 0 })
    }
}

impl From<String> for DynamicMessageNode {
    fn from(s: String) -> Self {
        Self::Bytes(s.into())
    }
}

impl From<DynamicMessage> for DynamicMessageNode {
    fn from(msg: DynamicMessage) -> Self {
        let b = msg.encode();
        Self::Bytes(b)
    }
}

macro_rules! into_node_impl {
    ($($x:ty as $variant:ident);* $(;)?) => {
        $(
        impl From<$x> for DynamicMessageNode {
            fn from(val: $x) -> Self {
                Self::$variant(val as _) // compiler will optimize this
            }
        }
        )*
    };
}

into_node_impl! {
    i32 as I64;
    u32 as U64;
    i64 as I64;
    u64 as U64;
    f32 as F32;
    f64 as F64;
    Bytes as Bytes;
    Vec<u64> as RepeatU64;
}

#[cfg(test)]
mod tests {
    use crate::dynamic_message;

    #[test]
    fn message() {
        let msg = dynamic_message! {
            1 => 23u32,
        }
        .encode();

        assert_eq!(*msg, [8, 23]);
    }
}
