pub mod db {
    use std::{
        collections::HashMap,
        time::{SystemTime, UNIX_EPOCH},
    };

    pub struct MinCache {
        buf: Vec<u8>,
        tail: u64,
        index_map: HashMap<String, u128>,
    }

    impl MinCache {
        pub fn new() -> Self {
            Self {
                buf: Vec::new(),
                tail: 0,
                index_map: HashMap::new(),
            }
        }

        pub fn put(&mut self, key: &String, value: &[u8]) {
            let time_stamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis();

            let entry_len = gen_entry(time_stamp, &key, value, &mut self.buf);

            let index = (self.tail as u128) << 64 | ((self.tail + entry_len) as u128);
            self.index_map.insert(key.clone(), index);
            self.tail += entry_len;
        }

        pub fn get(&mut self, key: &String) -> Option<String> {
            let index_option = self.index_map.get(key);

            let index = match index_option {
                None => {
                    return None;
                }
                Some(i) => i.clone(),
            };

            let end = (index & (0xffffffffffffffffu128)) as usize;
            let tail = (index >> 64) as usize;

            let slice = &self.buf[tail..end];

            match read_from_entry(key, slice) {
                None => None,
                Some(v) => String::from_utf8(v.to_vec()).ok(),
            }
        }
    }
    // timestamp keylen key value
    fn gen_entry(time_stamp: u128, key: &String, value: &[u8], buf: &mut Vec<u8>) -> u64 {
        let key_len = key.len() as u16;

        buf.extend_from_slice(&time_stamp.to_le_bytes()); // timestamp
        buf.extend_from_slice(&key_len.to_le_bytes()); // keylen
        buf.extend_from_slice(&key.as_bytes()); // key
        buf.extend_from_slice(value); // value

        let value_len = value.len() as u64;
        let entry_len = 16 + 2 + value_len + key_len as u64;

        entry_len
    }

    fn read_from_entry<'a>(key: &String, buf: &'a [u8]) -> Option<&'a [u8]> {
        let key_len = key.len();

        Some(&buf[16 + 2 + key_len..])
    }
}
