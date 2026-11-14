use std::cell::RefCell;
use std::hash::Hash;
use std::thread::LocalKey;

use lru::LruCache;
use tracing::trace;

pub struct Cache<K, V> {
    name: &'static str,
    lru: LruCache<K, V>,
    hit: u32,
    miss: u32,
    evict: u32,
}
impl<K, V> Cache<K, V>
where
    K: Hash + Eq,
    V: Clone,
{
    pub fn new(name: &'static str, size: usize) -> Self {
        Self {
            name,
            lru: LruCache::new(size.try_into().unwrap()),
            hit: 0,
            miss: 0,
            evict: 0,
        }
    }
    pub fn with<R>(
        thread_local: &'static LocalKey<RefCell<Cache<K, V>>>,
        compute: impl FnOnce(&mut Cache<K, V>) -> R,
    ) -> R {
        thread_local.with_borrow_mut(|cache| {
            let result = compute(cache);
            cache.report();
            result
        })
    }
    pub fn get_or_insert(&mut self, key: K, compute: impl Fn(&K, &mut Self) -> V) -> V {
        if let Some(value) = self.lru.get(&key) {
            self.hit += 1;
            value.clone()
        } else {
            self.miss += 1;
            let value = compute(&key, self);
            let evicted = self.lru.push(key, value.clone());
            if evicted.is_some() {
                self.evict += 1;
            }
            value
        }
    }
    fn report(&mut self) {
        trace!(
            "{} | hit={} miss={} evict={}",
            self.name, self.hit, self.miss, self.evict
        );
        self.hit = 0;
        self.miss = 0;
        self.evict = 0;
    }
}
