use core::error::Error;
use core::{convert::TryInto, time::Duration};

use alloc::borrow::ToOwned;
use alloc::boxed::Box;
use alloc::collections::{BTreeMap, BTreeSet, LinkedList};
use alloc::string::String;
use alloc::vec::Vec;

/// A trait for objects that can be serialized to binary.
pub trait SerBin {
    /// Serialize Self to bytes.
    ///
    /// This is a convenient wrapper around `ser_bin`.
    fn serialize_bin(&self) -> Vec<u8> {
        let mut s = Vec::new();
        self.ser_bin(&mut s);
        s
    }

    /// Serialize Self to bytes.
    ///
    /// ```rust
    /// # use nanoserde::*;
    /// let mut s = Vec::new();
    /// 42u32.ser_bin(&mut s);
    /// assert_eq!(s, vec![42, 0, 0, 0])
    /// ```
    fn ser_bin(&self, output: &mut Vec<u8>);
}

/// A trait for objects that can be deserialized from binary.
pub trait DeBin: Sized {
    /// Parse Self from the input bytes.
    ///
    /// This is a convenient wrapper around `de_bin`.
    fn deserialize_bin(d: &[u8]) -> Result<Self, DeBinErr> {
        DeBin::de_bin(&mut 0, d)
    }

    /// Parse Self from the input bytes starting at index `offset`.
    ///
    /// After deserialization, `offset` is updated to point at the byte after
    /// the last one used.
    ///
    /// ```rust
    /// # use nanoserde::*;
    /// let bytes = [1, 0, 0, 0, 2, 0, 0, 0];
    /// let mut offset = 4;
    /// let two = u32::de_bin(&mut offset, &bytes).unwrap();
    /// assert_eq!(two, 2);
    /// assert_eq!(offset, 8);
    /// ```
    fn de_bin(offset: &mut usize, bytes: &[u8]) -> Result<Self, DeBinErr>;
}

#[derive(Clone)]
#[non_exhaustive]
pub enum DeBinErrReason {
    Length {
        /// Expected Length
        expected_length: usize,
        /// Actual Length
        actual_length: usize,
    },
    Range(String),
}

/// The error message when failing to deserialize.
#[derive(Clone)]
pub struct DeBinErr {
    /// Offset
    pub o: usize,
    pub msg: DeBinErrReason,
}

impl DeBinErr {
    /// Helper for constructing [`DeBinErr`]
    pub fn new(offset: usize, expected_length: usize, actual_length: usize) -> Self {
        Self {
            o: offset,
            msg: DeBinErrReason::Length {
                expected_length,
                actual_length,
            },
        }
    }
}

impl core::fmt::Debug for DeBinErr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self.msg {
            DeBinErrReason::Length {
                expected_length: l,
                actual_length: s,
            } => write!(
                f,
                "Bin deserialize error at:{} wanted:{} bytes but max size is {}",
                self.o, l, s
            ),
            DeBinErrReason::Range(ref s) => write!(f, "Bin deserialize error at:{} {}", self.o, s),
        }
    }
}

impl core::fmt::Display for DeBinErr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(self, f)
    }
}

impl Error for DeBinErr {}

macro_rules! impl_ser_de_bin_for {
    ($ty:ident) => {
        impl SerBin for $ty {
            fn ser_bin(&self, s: &mut Vec<u8>) {
                let du8 = self.to_le_bytes();
                s.extend_from_slice(&du8);
            }
        }

        impl DeBin for $ty {
            fn de_bin(o: &mut usize, d: &[u8]) -> Result<$ty, DeBinErr> {
                let expected_length = core::mem::size_of::<$ty>();
                if *o + expected_length > d.len() {
                    return Err(DeBinErr {
                        o: *o,
                        msg: DeBinErrReason::Length {
                            expected_length,
                            actual_length: d.len(),
                        },
                    });
                }

                // We just checked that the correct amount of bytes are available,
                // and there are no invalid bit patterns for these primitives. This
                // unwrap should be impossible to hit.
                let ret: $ty =
                    <$ty>::from_le_bytes(d[*o..(*o + expected_length)].try_into().unwrap());
                *o += expected_length;
                Ok(ret)
            }
        }
    };
}

impl_ser_de_bin_for!(f64);
impl_ser_de_bin_for!(f32);
impl_ser_de_bin_for!(u128);
impl_ser_de_bin_for!(i128);
impl_ser_de_bin_for!(u64);
impl_ser_de_bin_for!(i64);
impl_ser_de_bin_for!(u32);
impl_ser_de_bin_for!(i32);
impl_ser_de_bin_for!(u16);
impl_ser_de_bin_for!(i16);
impl_ser_de_bin_for!(i8);

impl SerBin for usize {
    fn ser_bin(&self, s: &mut Vec<u8>) {
        let u64usize = *self as u64;
        let du8 = u64usize.to_le_bytes();
        s.extend_from_slice(&du8);
    }
}

impl DeBin for usize {
    fn de_bin(o: &mut usize, d: &[u8]) -> Result<usize, DeBinErr> {
        let l = core::mem::size_of::<u64>();

        let m = match d.get(*o..*o + l) {
            Some(data) => u64::from_le_bytes(data.try_into().unwrap()),
            None => {
                return Err(DeBinErr {
                    o: *o,
                    msg: DeBinErrReason::Length {
                        expected_length: l,
                        actual_length: d.len(),
                    },
                });
            }
        };

        *o += l;
        Ok(m as usize)
    }
}

impl DeBin for u8 {
    fn de_bin(o: &mut usize, d: &[u8]) -> Result<u8, DeBinErr> {
        if *o + 1 > d.len() {
            return Err(DeBinErr {
                o: *o,
                msg: DeBinErrReason::Length {
                    expected_length: 1,
                    actual_length: d.len(),
                },
            });
        }
        let m = d[*o];
        *o += 1;
        Ok(m)
    }
}

impl SerBin for u8 {
    fn ser_bin(&self, s: &mut Vec<u8>) {
        s.push(*self);
    }
}

impl SerBin for bool {
    fn ser_bin(&self, s: &mut Vec<u8>) {
        s.push(if *self { 1 } else { 0 });
    }
}

impl DeBin for bool {
    fn de_bin(o: &mut usize, d: &[u8]) -> Result<bool, DeBinErr> {
        if *o + 1 > d.len() {
            return Err(DeBinErr {
                o: *o,
                msg: DeBinErrReason::Length {
                    expected_length: 1,
                    actual_length: d.len(),
                },
            });
        }
        let m = d[*o];
        *o += 1;
        if m == 0 {
            Ok(false)
        } else {
            Ok(true)
        }
    }
}

impl SerBin for String {
    fn ser_bin(&self, s: &mut Vec<u8>) {
        let len = self.len();
        len.ser_bin(s);
        s.extend_from_slice(self.as_bytes());
    }
}

impl DeBin for String {
    fn de_bin(o: &mut usize, d: &[u8]) -> Result<String, DeBinErr> {
        let len: usize = DeBin::de_bin(o, d)?;
        if *o + len > d.len() {
            return Err(DeBinErr {
                o: *o,
                msg: DeBinErrReason::Length {
                    expected_length: 1,
                    actual_length: d.len(),
                },
            });
        }
        let r = match core::str::from_utf8(&d[*o..(*o + len)]) {
            Ok(r) => r.to_owned(),
            Err(_) => {
                return Err(DeBinErr {
                    o: *o,
                    msg: DeBinErrReason::Length {
                        expected_length: len,
                        actual_length: d.len(),
                    },
                })
            }
        };
        *o += len;
        Ok(r)
    }
}

impl<T> SerBin for Vec<T>
where
    T: SerBin,
{
    fn ser_bin(&self, s: &mut Vec<u8>) {
        let len = self.len();
        len.ser_bin(s);
        for item in self {
            item.ser_bin(s);
        }
    }
}

impl<T> DeBin for Vec<T>
where
    T: DeBin,
{
    fn de_bin(o: &mut usize, d: &[u8]) -> Result<Vec<T>, DeBinErr> {
        let len: usize = DeBin::de_bin(o, d)?;
        let mut out = Vec::with_capacity(len);
        for _ in 0..len {
            out.push(DeBin::de_bin(o, d)?)
        }
        Ok(out)
    }
}

impl<T> SerBin for LinkedList<T>
where
    T: SerBin,
{
    fn ser_bin(&self, s: &mut Vec<u8>) {
        let len = self.len();
        len.ser_bin(s);
        for item in self.iter() {
            item.ser_bin(s);
        }
    }
}

impl<T> DeBin for LinkedList<T>
where
    T: DeBin,
{
    fn de_bin(o: &mut usize, d: &[u8]) -> Result<LinkedList<T>, DeBinErr> {
        let len: usize = DeBin::de_bin(o, d)?;
        let mut out = LinkedList::new();
        for _ in 0..len {
            out.push_back(DeBin::de_bin(o, d)?)
        }
        Ok(out)
    }
}

#[cfg(feature = "std")]
impl<T> SerBin for std::collections::HashSet<T>
where
    T: SerBin,
{
    fn ser_bin(&self, s: &mut Vec<u8>) {
        let len = self.len();
        len.ser_bin(s);
        for item in self.iter() {
            item.ser_bin(s);
        }
    }
}

#[cfg(feature = "std")]
impl<T> DeBin for std::collections::HashSet<T>
where
    T: DeBin + core::hash::Hash + Eq,
{
    fn de_bin(o: &mut usize, d: &[u8]) -> Result<Self, DeBinErr> {
        let len: usize = DeBin::de_bin(o, d)?;
        let mut out = std::collections::HashSet::with_capacity(len);
        for _ in 0..len {
            out.insert(DeBin::de_bin(o, d)?);
        }
        Ok(out)
    }
}

impl<T> SerBin for BTreeSet<T>
where
    T: SerBin,
{
    fn ser_bin(&self, s: &mut Vec<u8>) {
        let len = self.len();
        len.ser_bin(s);
        for item in self.iter() {
            item.ser_bin(s);
        }
    }
}

impl<T> DeBin for BTreeSet<T>
where
    T: DeBin + Ord,
{
    fn de_bin(o: &mut usize, d: &[u8]) -> Result<BTreeSet<T>, DeBinErr> {
        let len: usize = DeBin::de_bin(o, d)?;
        let mut out = BTreeSet::new();
        for _ in 0..len {
            out.insert(DeBin::de_bin(o, d)?);
        }
        Ok(out)
    }
}

impl<T> SerBin for Option<T>
where
    T: SerBin,
{
    fn ser_bin(&self, s: &mut Vec<u8>) {
        if let Some(v) = self {
            s.push(1);
            v.ser_bin(s);
        } else {
            s.push(0);
        }
    }
}

impl<T> DeBin for Option<T>
where
    T: DeBin,
{
    fn de_bin(o: &mut usize, d: &[u8]) -> Result<Option<T>, DeBinErr> {
        if *o + 1 > d.len() {
            return Err(DeBinErr {
                o: *o,
                msg: DeBinErrReason::Length {
                    expected_length: 1,
                    actual_length: d.len(),
                },
            });
        }
        let m = d[*o];
        *o += 1;
        if m == 1 {
            Ok(Some(DeBin::de_bin(o, d)?))
        } else {
            Ok(None)
        }
    }
}

impl<T> SerBin for [T]
where
    T: SerBin,
{
    fn ser_bin(&self, s: &mut Vec<u8>) {
        for item in self {
            item.ser_bin(s);
        }
    }
}

impl<T, const N: usize> SerBin for [T; N]
where
    T: SerBin,
{
    #[inline(always)]
    fn ser_bin(&self, s: &mut Vec<u8>) {
        self.as_slice().ser_bin(s)
    }
}

impl<T, const N: usize> DeBin for [T; N]
where
    T: DeBin,
{
    fn de_bin(o: &mut usize, d: &[u8]) -> Result<Self, DeBinErr> {
        use core::mem::MaybeUninit;

        // waiting for uninit_array(or for array::try_from_fn) stabilization
        // https://github.com/rust-lang/rust/issues/96097
        // https://github.com/rust-lang/rust/issues/89379
        let mut to: [MaybeUninit<T>; N] =
            unsafe { MaybeUninit::<[MaybeUninit<T>; N]>::uninit().assume_init() };

        for index in 0..N {
            to[index] = match DeBin::de_bin(o, d) {
                Ok(v) => MaybeUninit::new(v),
                Err(e) => {
                    // drop all the MaybeUninit values which we've already
                    // successfully deserialized so we don't leak memory.
                    // See https://github.com/not-fl3/nanoserde/issues/79
                    for (_, to_drop) in (0..index).zip(to) {
                        unsafe { to_drop.assume_init() };
                    }
                    return Err(e);
                }
            }
        }

        // waiting for array_assume_init or core::array::map optimizations
        // https://github.com/rust-lang/rust/issues/61956
        Ok(unsafe { (*(&to as *const _ as *const MaybeUninit<_>)).assume_init_read() })
    }
}

impl SerBin for () {
    #[inline(always)]
    fn ser_bin(&self, _s: &mut Vec<u8>) {
        // do nothing
    }
}

impl DeBin for () {
    #[inline(always)]
    fn de_bin(_o: &mut usize, _d: &[u8]) -> Result<Self, DeBinErr> {
        Ok(())
    }
}

impl<A, B> SerBin for (A, B)
where
    A: SerBin,
    B: SerBin,
{
    fn ser_bin(&self, s: &mut Vec<u8>) {
        self.0.ser_bin(s);
        self.1.ser_bin(s);
    }
}

impl<A, B> DeBin for (A, B)
where
    A: DeBin,
    B: DeBin,
{
    fn de_bin(o: &mut usize, d: &[u8]) -> Result<(A, B), DeBinErr> {
        Ok((DeBin::de_bin(o, d)?, DeBin::de_bin(o, d)?))
    }
}

impl<A, B, C> SerBin for (A, B, C)
where
    A: SerBin,
    B: SerBin,
    C: SerBin,
{
    fn ser_bin(&self, s: &mut Vec<u8>) {
        self.0.ser_bin(s);
        self.1.ser_bin(s);
        self.2.ser_bin(s);
    }
}

impl<A, B, C> DeBin for (A, B, C)
where
    A: DeBin,
    B: DeBin,
    C: DeBin,
{
    fn de_bin(o: &mut usize, d: &[u8]) -> Result<(A, B, C), DeBinErr> {
        Ok((
            DeBin::de_bin(o, d)?,
            DeBin::de_bin(o, d)?,
            DeBin::de_bin(o, d)?,
        ))
    }
}

impl<A, B, C, D> SerBin for (A, B, C, D)
where
    A: SerBin,
    B: SerBin,
    C: SerBin,
    D: SerBin,
{
    fn ser_bin(&self, s: &mut Vec<u8>) {
        self.0.ser_bin(s);
        self.1.ser_bin(s);
        self.2.ser_bin(s);
        self.3.ser_bin(s);
    }
}

impl<A, B, C, D> DeBin for (A, B, C, D)
where
    A: DeBin,
    B: DeBin,
    C: DeBin,
    D: DeBin,
{
    fn de_bin(o: &mut usize, d: &[u8]) -> Result<(A, B, C, D), DeBinErr> {
        Ok((
            DeBin::de_bin(o, d)?,
            DeBin::de_bin(o, d)?,
            DeBin::de_bin(o, d)?,
            DeBin::de_bin(o, d)?,
        ))
    }
}

#[cfg(feature = "std")]
impl<K, V> SerBin for std::collections::HashMap<K, V>
where
    K: SerBin,
    V: SerBin,
{
    fn ser_bin(&self, s: &mut Vec<u8>) {
        let len = self.len();
        len.ser_bin(s);
        for (k, v) in self {
            k.ser_bin(s);
            v.ser_bin(s);
        }
    }
}

#[cfg(feature = "std")]
impl<K, V> DeBin for std::collections::HashMap<K, V>
where
    K: DeBin + core::cmp::Eq + core::hash::Hash,
    V: DeBin,
{
    fn de_bin(o: &mut usize, d: &[u8]) -> Result<Self, DeBinErr> {
        let len: usize = DeBin::de_bin(o, d)?;
        let mut h = std::collections::HashMap::with_capacity(len);
        for _ in 0..len {
            let k = DeBin::de_bin(o, d)?;
            let v = DeBin::de_bin(o, d)?;
            h.insert(k, v);
        }
        Ok(h)
    }
}

impl<K, V> SerBin for BTreeMap<K, V>
where
    K: SerBin,
    V: SerBin,
{
    fn ser_bin(&self, s: &mut Vec<u8>) {
        let len = self.len();
        len.ser_bin(s);
        for (k, v) in self {
            k.ser_bin(s);
            v.ser_bin(s);
        }
    }
}

impl<K, V> DeBin for BTreeMap<K, V>
where
    K: DeBin + core::cmp::Eq + Ord,
    V: DeBin,
{
    fn de_bin(o: &mut usize, d: &[u8]) -> Result<Self, DeBinErr> {
        let len: usize = DeBin::de_bin(o, d)?;
        let mut h = BTreeMap::new();
        for _ in 0..len {
            let k = DeBin::de_bin(o, d)?;
            let v = DeBin::de_bin(o, d)?;
            h.insert(k, v);
        }
        Ok(h)
    }
}

impl<T> SerBin for Box<T>
where
    T: SerBin,
{
    fn ser_bin(&self, s: &mut Vec<u8>) {
        (**self).ser_bin(s)
    }
}

impl<T> DeBin for Box<T>
where
    T: DeBin,
{
    fn de_bin(o: &mut usize, d: &[u8]) -> Result<Box<T>, DeBinErr> {
        Ok(Box::new(DeBin::de_bin(o, d)?))
    }
}

impl SerBin for Duration {
    fn ser_bin(&self, s: &mut Vec<u8>) {
        let secs = self.as_secs();
        let nanos = self.subsec_nanos();
        secs.ser_bin(s);
        nanos.ser_bin(s);
    }
}

impl DeBin for Duration {
    fn de_bin(o: &mut usize, d: &[u8]) -> Result<Duration, DeBinErr> {
        let secs: u64 = DeBin::de_bin(o, d)?;
        let nanos: u32 = DeBin::de_bin(o, d)?;
        if nanos > 1_000_000_000 {
            return Err(DeBinErr {
                o: *o,
                msg: DeBinErrReason::Range(
                    "Duration nanos must be at most 1,000,000,000".to_owned(),
                ),
            });
        }
        Ok(Duration::new(secs, nanos))
    }
}

#[cfg(feature = "std")]
impl SerBin for std::time::SystemTime {
    fn ser_bin(&self, s: &mut Vec<u8>) {
        let duration = self.duration_since(std::time::SystemTime::UNIX_EPOCH).ok();
        duration.ser_bin(s);
    }
}

#[cfg(feature = "std")]
impl DeBin for std::time::SystemTime {
    fn de_bin(o: &mut usize, d: &[u8]) -> Result<std::time::SystemTime, DeBinErr> {
        match DeBin::de_bin(o, d)? {
            Some(duration) => Ok(std::time::SystemTime::UNIX_EPOCH + duration),
            None => Ok(std::time::SystemTime::UNIX_EPOCH),
        }
    }
}
