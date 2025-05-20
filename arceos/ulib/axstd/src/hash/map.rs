use super::default_hasher::DefaultHasher;
extern crate alloc;
use alloc::vec::Vec;
use core::cmp::Eq;
use core::option::Option::Some;
use core::{
    hash::{Hash, Hasher},
    ops::{Index, IndexMut},
};
use core::iter::IntoIterator;
const ININIAL_NBUCKETS: usize = 1;
pub struct HashMap<K, V> {
    buckets: Vec<Vec<(K, V)>>,
    item: usize,
}
impl<K, V> HashMap<K, V> {
    pub fn new() -> Self {
        HashMap {
            buckets: Vec::new(),
            item: 0,
        }
    }
}
impl<K, V> HashMap<K, V>
where
    K: Hash + Eq,
{
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if self.buckets.is_empty() || self.item > 3 * self.buckets.len() / 4 {
            self.resize();
        }
        let index = self.bucket(&key);
        let bucket = &mut self.buckets[index];

        // 对可变引用执行
        // if in bucket就是移动所有需要使用.iter_mut()
        for &mut (ref ekey, ref mut evalue) in bucket.iter_mut() {
            // 等于检查
            // 如果类型 K 实现了 Eq trait，那么对于引用类型 &K，它也会自动实现 Eq trait。
            // 这是因为在 Rust 中，对于实现了 Eq 的类型，其引用类型会自动派生（derive）Eq trait 的实现。
            if ekey == &key {
                use core::mem;
                // 告诉用户旧数据
                return Some(mem::replace(evalue, value));
            }
        }
        self.item += 1;
        bucket.push((key, value));
        None
    }
    fn resize(&mut self) {
        let target_size = match self.buckets.len() {
            0 => ININIAL_NBUCKETS,
            n => 2 * n,
        };
        // 这种方法会在迭代过程中动态地分配内存，并将每个 Vec::new() 的结果添加到
        let mut new_buckets = Vec::with_capacity(target_size);
        // 用于将一个迭代器的元素取到集合中
        new_buckets.extend((0..target_size).map(|_| Vec::new()));
        for (k, v) in self.buckets.iter_mut().flat_map(|bucket| bucket.drain(..)) {
            let mut hasher = DefaultHasher::new();
            k.hash(&mut hasher);
            let bucket = (hasher.finish() % new_buckets.len() as u64) as usize;
            new_buckets[bucket].push((k, v));
        }
        self.buckets = new_buckets;
    }

    pub fn bucket(&self, key: &K) -> usize {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        (hasher.finish() % self.buckets.len() as u64) as usize
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let index = self.bucket(key);
        self.buckets[index]
            .iter()
            .find(|&&(ref x, _)| x == key)
            .map(|&(_, ref v)| v)
    }

    pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
        let index = self.bucket(key);
        self.buckets[index]
            .iter_mut()
            .find(|&&mut (ref x, _)| x == key)
            .map(|&mut (_, ref mut v)| v)
    }

    pub fn contains_key(&self, key: &K) -> bool {
        let index = self.bucket(key);
        self.buckets[index]
            .iter()
            .find(|&&(ref x, _)| x == key)
            .is_some()
    }

    pub fn remove(&mut self, key: &K) -> Option<V> {
        let index = self.bucket(key);
        let bucket = &mut self.buckets[index];
        let i = bucket.iter().position(|&(ref x, _)| x == key)?;
        // remove会移除所有元素
        self.item -= 1;
        Some(bucket.swap_remove(i).1)
    }
    pub fn len(&self) -> usize {
        self.item
    }

    pub fn is_empty(&self) -> bool {
        self.item == 0
    }
}
pub struct Iter<'a, K, V> {
    map: &'a HashMap<K, V>,
    bucket: usize,
    at: usize,
}

impl<'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.map.buckets.get(self.bucket) {
                Some(bucket) => match bucket.get(self.at) {
                    Some(&(ref k, ref v)) => {
                        self.at += 1;
                        break Some((k, v));
                    }
                    None => {
                        self.bucket += 1;
                        self.at = 0;
                        continue;
                    }
                },
                None => break None,
            }
        }
    }
}

impl<K, V> HashMap<K, V> {
    pub fn iter(&self) -> Iter::<K, V> {
        Iter::<K, V> {
            map: &self,
            bucket: 0,
            at: 0,
        }
    }
}

impl<K, V> Index<K> for HashMap<K, V>
where
    K: core::hash::Hash + Eq,
{
    type Output = V;

    fn index(&self, key: K) -> &Self::Output {
        self.get(&key).expect("Key not found")
    }
}

impl<K, V> IndexMut<K> for HashMap<K, V>
where
    K: core::hash::Hash + Eq,
{
    fn index_mut(&mut self, key: K) -> &mut Self::Output {
        self.get_mut(&key).expect("Key not found")
    }
}

#[test]
#[cfg(test)]
mod tests {
    use super::*;
    use core::assert_eq;
    use core::option::Option::Some;
    #[test]
    fn insert() {
        let mut map = HashMap::new();
        assert_eq!(map.is_empty(), true);
        map.insert("key", "value");
        assert_eq!(map.get(&"key"), Some(&"value1"));
        assert_eq!(map.remove(&"key"), Some("value1"));
        assert_eq!(map.get(&"key"), None);
        assert_eq!(map.len(), 1);
    }
}
