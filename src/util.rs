// Utilities for threadsafe atomic variables. Mostly wrappers to make the primitive types easier to use
use std::sync::atomic::{self, Ordering};

pub struct AtomicBool(atomic::AtomicBool);
impl AtomicBool {
    pub fn new(v: bool) -> AtomicBool {
        AtomicI8(atomic::AtomicBool::new(v))
    }
    pub fn get(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
    pub fn set(&self, v: bool) {
        self.0.store(v, Ordering::Relaxed)
    }
}
/// Simple wrapper over `AtomicI8` with relaxed ordering.
pub struct AtomicI8(atomic::AtomicI8);
#[allow(dead_code)]
impl AtomicI8 {
    /// Create a new atomic 8-bit integer with initial value `v`.
    pub fn new(v: i8) -> AtomicI8 {
        AtomicI8(atomic::AtomicI8::new(v))
    }
    /// Loads a value from the atomic integer with relaxed ordering.
    pub fn get(&self) -> i8 {
        self.0.load(Ordering::Relaxed)
    }
    /// Stores a value into the atomic integer with relaxed ordering.
    pub fn set(&self, v: i8) {
        self.0.store(v, Ordering::Relaxed)
    }
}
/// Simple wrapper over `AtomicUsize` with relaxed ordering.
pub struct AtomicUsize(atomic::AtomicUsize);
#[allow(dead_code)]
impl AtomicUsize {
    /// Create a new atomic integer with initial value `v`.
    pub fn new(v: usize) -> AtomicUsize {
        AtomicUsize(atomic::AtomicUsize::new(v))
    }
    /// Loads a value from the atomic integer with relaxed ordering.
    pub fn get(&self) -> usize {
        self.0.load(Ordering::Relaxed)
    }
    /// Stores a value into the atomic integer with relaxed ordering.
    pub fn set(&self, v: usize) {
        self.0.store(v, Ordering::Relaxed)
    }
}
/// Simple 32-bit floating point wrapper over `AtomicU32` with relaxed ordering.
pub struct AtomicF32(atomic::AtomicU32);
#[allow(dead_code)]
impl AtomicF32 {
    /// Create a new atomic 32-bit float with initial value `v`.
    pub fn new(v: f32) -> AtomicF32 {
        AtomicF32(atomic::AtomicU32::new(v.to_bits()))
    }

    /// Loads a value from the atomic float with relaxed ordering.
    pub fn get(&self) -> f32 {
        f32::from_bits(self.0.load(Ordering::Relaxed))
    }
    /// Stores a value into the atomic float with relaxed ordering.
    pub fn set(&self, v: f32) {
        self.0.store(v.to_bits(), Ordering::Relaxed)
    }
}