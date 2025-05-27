use crate::db::db::MinCache;
mod db;

fn main() {
    let mut cache = MinCache::new();
    full_entry(&mut cache);
    let result = cache.get(&String::from("name"));
    println!("{:?}", result);
}

fn full_entry(cache: &mut MinCache) {
    let key1 = String::from("key1");
    let value1 = String::from("my value1");
    let key2 = String::from("key2");
    let value2 = String::from("my value2");
    let name = String::from("name");
    let name_value = String::from("张三");

    cache.put(&key1, value1.as_bytes());
    cache.put(&key2, value2.as_bytes());
    cache.put(&name, name_value.as_bytes());
}
