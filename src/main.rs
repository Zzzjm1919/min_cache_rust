pub mod db {
    use std::{
        // collections::HashMap,
        time::{SystemTime, UNIX_EPOCH},
    };

    pub struct MinCache {
        buf: Vec<u8>,
        // tail: u64,
        // index_map: HashMap<String, u64>,
    }

    impl MinCache {
        pub fn new() -> Self {
            Self {
                buf: Vec::new(),
                // tail: 0,
                // index_map: HashMap::new(),
            }
        }

        pub fn put(&mut self, key: String, value: &[u8]) {
            let time_stamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis();

            gen_entry(time_stamp, key, value, &mut self.buf)
        }

        pub fn get(&mut self, key: &str) -> Option<String> {
            let slice = &self.buf[16 + 2..];
            match read_from_entry(key, slice) {
                None => None,
                Some(v) => String::from_utf8(v.to_vec()).ok(),
            }
        }
    }

    fn gen_entry(time_stamp: u128, key: String, value: &[u8], buf: &mut Vec<u8>) {
        let key_len = key.len() as u16;
        buf.extend_from_slice(&time_stamp.to_le_bytes());

        buf.extend_from_slice(&key_len.to_le_bytes());
        buf.extend_from_slice(&key.as_bytes());
        buf.extend_from_slice(value);
    }

    fn read_from_entry<'a>(key: &str, buf: &'a [u8]) -> Option<&'a [u8]> {
        let key_len = key.len();

        Some(&buf[key_len..])
    }
}

fn main() {
    let mut cache = db::MinCache::new();
    let value = String::from("my value");
    cache.put(String::from("key1"), value.as_bytes());

    let result = cache.get("key1");
    println!("{:?}", result);
}
