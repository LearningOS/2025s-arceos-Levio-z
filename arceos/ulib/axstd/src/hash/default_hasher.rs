use arceos_api::mem::ax_random;
use core::{hash::Hasher};
use siphasher::sip::SipHasher13;

pub struct DefaultHasher(SipHasher13);
impl DefaultHasher {
    pub fn new() -> DefaultHasher {
        DefaultHasher(SipHasher13::new_with_key(&ax_random().to_ne_bytes()))
    }
}
impl Hasher for DefaultHasher {
    // 作用：向哈希器写入字节数据（&[u8]），更新哈希状态。
    #[inline]
    fn write(&mut self, msg: &[u8]) {
        self.0.write(msg)
    }

    #[inline]
    // 结束哈希过程，返回最终的 u64 哈希值。
    fn finish(&self) -> u64 {
        self.0.finish()
    }
}