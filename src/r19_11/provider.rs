use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicBool, Ordering};

// ========================================
// より安全な実装
// ========================================

pub trait TimestampProvider: Send + Sync {
    fn get_timestamp(&self) -> u32;
}

pub trait SessionIdProvider: Send + Sync {
    fn get_session_id(&self) -> u32;
}

// 2. 静的 Provider の実装
pub struct StaticTimestampProvider {
    get_fn: fn() -> u32,
}

impl StaticTimestampProvider {
    pub const fn new(get_fn: fn() -> u32) -> Self {
        Self { get_fn }
    }
}

impl TimestampProvider for StaticTimestampProvider {
    fn get_timestamp(&self) -> u32 {
        (self.get_fn)()
    }
}

pub struct StaticSessionIdProvider {
    get_fn: fn() -> u32,
}

impl StaticSessionIdProvider {
    pub const fn new(get_fn: fn() -> u32) -> Self {
        Self { get_fn }
    }
}

impl SessionIdProvider for StaticSessionIdProvider {
    fn get_session_id(&self) -> u32 {
        (self.get_fn)()
    }
}

pub struct GlobalProvider<T: ?Sized + 'static> {
    initialized: AtomicBool,
    provider: UnsafeCell<Option<&'static T>>,
}

unsafe impl<T: ?Sized + Sync> Sync for GlobalProvider<T> {}

impl<T: ?Sized> GlobalProvider<T> {
    pub const fn new() -> Self {
        Self {
            initialized: AtomicBool::new(false),
            provider: UnsafeCell::new(None),
        }
    }

    pub fn set(&self, provider: &'static T) {
        if self.initialized.swap(true, Ordering::SeqCst) {
            panic!("Provider already initialized");
        }
        unsafe {
            *self.provider.get() = Some(provider);
        }
    }

    pub fn get(&self) -> Option<&'static T> {
        if self.initialized.load(Ordering::SeqCst) {
            unsafe { *self.provider.get() }
        } else {
            None
        }
    }
}

pub static GLOBAL_TIMESTAMP: GlobalProvider<dyn TimestampProvider> = 
    GlobalProvider::new();

pub static GLOBAL_SESSION: GlobalProvider<dyn SessionIdProvider> = 
    GlobalProvider::new();

pub fn set_global_timestamp_provider(provider: &'static dyn TimestampProvider) {
    GLOBAL_TIMESTAMP.set(provider);
}

pub fn set_global_session_id_provider(provider: &'static dyn SessionIdProvider) {
    GLOBAL_SESSION.set(provider);
}

