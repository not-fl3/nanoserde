//! Regression tests: untrusted length prefixes in DeBin must not drive
//! large allocations (CWE-770) or panic via integer overflow (CWE-680).
//!
//! The counting `#[global_allocator]` is process-global, so all checks are
//! merged into a single `#[test]` to keep them serial and unpolluted.
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

static PEAK: AtomicUsize = AtomicUsize::new(0);
static CURRENT: AtomicUsize = AtomicUsize::new(0);

struct Counting;

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = System.alloc(layout);
        if !ptr.is_null() {
            CURRENT.fetch_add(layout.size(), Ordering::SeqCst);
            PEAK.fetch_max(CURRENT.load(Ordering::SeqCst), Ordering::SeqCst);
        }
        ptr
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
        CURRENT.fetch_sub(layout.size(), Ordering::SeqCst);
    }
    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let new_ptr = System.realloc(ptr, layout, new_size);
        if !new_ptr.is_null() {
            CURRENT.fetch_sub(layout.size(), Ordering::SeqCst);
            CURRENT.fetch_add(new_size, Ordering::SeqCst);
            PEAK.fetch_max(CURRENT.load(Ordering::SeqCst), Ordering::SeqCst);
        }
        new_ptr
    }
}

#[global_allocator]
static A: Counting = Counting;

use nanoserde::DeBin;

/// An 8-byte declared length (64 MiB) with no element data must error without
/// preallocating 64 MiB in Vec / HashSet / HashMap, and an overflowing String
/// length must return Err instead of panicking.
#[test]
fn untrusted_length_prefix_is_bounded() {
    let declared: u64 = 64 * 1024 * 1024;
    let data = declared.to_le_bytes().to_vec(); // 8 bytes, truncated

    // Vec<u8>
    PEAK.store(0, Ordering::SeqCst);
    assert!(Vec::<u8>::de_bin(&mut 0, &data).is_err());
    let vec_peak = PEAK.load(Ordering::SeqCst);
    assert!(
        vec_peak < 8192,
        "Vec must grow from actual data, not from the declared length (peak {vec_peak} bytes)"
    );

    // HashMap<u8, u8>
    PEAK.store(0, Ordering::SeqCst);
    assert!(std::collections::HashMap::<u8, u8>::de_bin(&mut 0, &data).is_err());
    let map_peak = PEAK.load(Ordering::SeqCst);
    assert!(
        map_peak < 8192,
        "HashMap must grow from actual data, not from the declared length (peak {map_peak} bytes)"
    );

    // HashSet<u8>
    PEAK.store(0, Ordering::SeqCst);
    assert!(std::collections::HashSet::<u8>::de_bin(&mut 0, &data).is_err());
    let set_peak = PEAK.load(Ordering::SeqCst);
    assert!(
        set_peak < 8192,
        "HashSet must grow from actual data, not from the declared length (peak {set_peak} bytes)"
    );

    // String: `*o + len` used to overflow (o=1, len=usize::MAX) and panic on the slice.
    let mut data = vec![0xAAu8];
    data.extend((usize::MAX as u64).to_le_bytes());
    let r = std::panic::catch_unwind(|| String::de_bin(&mut 1, &data));
    assert!(
        matches!(r, Ok(Err(_))),
        "overflowing String length must return Err, not panic"
    );
}
