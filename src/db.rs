pub mod db {
    use std::{
        collections::HashMap,
        // sync::{Arc, RwLock},
        time::{Duration, SystemTime, UNIX_EPOCH},
    };

    pub struct MinCache {
        buf: Vec<u8>,
        tail: u64,
        index_map: HashMap<String, u128>,
        // lock: RwLock<i8>,
    }

    impl MinCache {
        // 无过期时间
        const NEVER_EXPIRE: u128 = 1 << 127;

        pub fn new() -> Self {
            Self {
                buf: Vec::new(),
                tail: 0,
                index_map: HashMap::new(),
                // lock: RwLock::new(0),
            }
        }

        pub fn put(&mut self, key: &String, value: &[u8]) {
            let entry_len = gen_entry(Self::NEVER_EXPIRE, &key, value, &mut self.buf);

            let index = (self.tail as u128) << 64 | ((self.tail + entry_len) as u128);
            self.index_map.insert(key.clone(), index);
            self.tail += entry_len;
        }

        pub fn put_with_ttl(&mut self, key: &String, value: &[u8], ttl: u128) {
            let exp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis()
                + (ttl * 1000);

            let entry_len = gen_entry(exp, &key, value, &mut self.buf);

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

        let time_stamp_buf = &buf[..16];

        let time_stamp = u128::from_le_bytes(time_stamp_buf.try_into().unwrap());

        let exp_time = UNIX_EPOCH + Duration::from_millis(time_stamp as u64);
        if exp_time.lt(&SystemTime::now()) && time_stamp != MinCache::NEVER_EXPIRE {
            return None;
        }
        Some(&buf[16 + 2 + key_len..])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_put_and_get_single_entry() {
        let mut cache = db::MinCache::new();
        let key = String::from("test_key");
        let value = String::from("test_value");

        cache.put(&key, value.as_bytes());

        let result = cache.get(&key);
        assert_eq!(result, Some(value));
    }

    #[test]
    fn test_put_multiple_entries_and_get() {
        let mut cache = db::MinCache::new();
        let kvs = vec![
            (String::from("key1"), String::from("val1")),
            (String::from("key2"), String::from("val2")),
            (String::from("中文键"), String::from("值123")),
        ];

        for (k, v) in &kvs {
            cache.put(k, v.as_bytes());
        }

        for (k, v) in &kvs {
            let result = cache.get(k);
            assert_eq!(result, Some(v.clone()));
        }
    }

    #[test]
    fn test_get_nonexistent_key_returns_none() {
        let mut cache = db::MinCache::new();
        cache.put(&String::from("exists"), b"value");

        let result = cache.get(&String::from("missing"));
        assert_eq!(result, None);
    }

    #[test]
    fn test_empty_cache_returns_none() {
        let mut cache = db::MinCache::new();
        let result = cache.get(&String::from("anykey"));
        assert_eq!(result, None);
    }
}
