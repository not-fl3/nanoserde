use core::convert::TryInto;
use core::hash::Hash;

use alloc::boxed::Box;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

#[cfg(features = "no_std")]
use hashbrown::HashMap;

#[cfg(not(features = "no_std"))]
use std::collections::HashMap;
use core::hash::Hash;

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

/// The error message when failing to deserialize from raw bytes.
#[derive(Clone)]
pub struct DeBinErr {
    pub o: usize,
    pub l: usize,
    pub s: usize,
}

impl core::fmt::Debug for DeBinErr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Bin deserialize error at:{} wanted:{} bytes but max size is {}",
            self.o, self.l, self.s
        )
    }
}

impl core::fmt::Display for DeBinErr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(self, f)
    }
}

#[cfg(features = "no_std")]
impl core::error::Error for DeBinErr {}

#[cfg(not(features = "no_std"))]
impl std::error::Error for DeBinErr {}

macro_rules! impl_ser_de_bin_for {
    ($ty:ident) => {
        impl SerBin for $ty {
            fn ser_bin(&self, s: &mut Vec<u8>) {
                let du8 = self.to_ne_bytes();
                s.extend_from_slice(&du8);
            }
        }

        impl DeBin for $ty {
            fn de_bin(o: &mut usize, d: &[u8]) -> Result<$ty, DeBinErr> {
                let l = core::mem::size_of::<$ty>();
                if *o + l > d.len() {
                    return Err(DeBinErr {
                        o: *o,
                        l: l,
                        s: d.len(),
                    });
                }
                let mut m = [0 as $ty];
                m[0] = <$ty>::from_ne_bytes(d[*o..(*o + l)].try_into().unwrap());
                *o += l;
                Ok(m[0])
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

impl SerBin for usize {
    fn ser_bin(&self, s: &mut Vec<u8>) {
        let u64usize = *self as u64;
        let du8 = u64usize.to_ne_bytes();
        s.extend_from_slice(&du8);
    }
}

impl DeBin for usize {
    fn de_bin(o: &mut usize, d: &[u8]) -> Result<usize, DeBinErr> {
        let l = core::mem::size_of::<u64>();

        let m = match d.get(*o..*o + l) {
            Some(data) => u64::from_ne_bytes(data.try_into().unwrap()),
            None => {
                return Err(DeBinErr {
                    o: *o,
                    l,
                    s: d.len(),
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
                l: 1,
                s: d.len(),
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
                l: 1,
                s: d.len(),
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
                l: 1,
                s: d.len(),
            });
        }
        let r = core::str::from_utf8(&d[*o..(*o + len)])
            .unwrap()
            .to_string();
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
        let mut out = Vec::new();
        for _ in 0..len {
            out.push(DeBin::de_bin(o, d)?)
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
                l: 1,
                s: d.len(),
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

impl<T, const N: usize> DeBin for [T; N]
where
    T: DeBin,
{
    fn de_bin(o: &mut usize, d: &[u8]) -> Result<Self, DeBinErr> {
        unsafe {
            let mut to = core::mem::MaybeUninit::<[T; N]>::uninit();
            let top: *mut T = core::mem::transmute(&mut to);
            for c in 0..N {
                top.add(c).write(DeBin::de_bin(o, d)?);
            }
            Ok(to.assume_init())
        }
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

impl<K, V> SerBin for HashMap<K, V>
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

impl<K, V> DeBin for HashMap<K, V>
where
    K: DeBin + Eq + Hash,
    V: DeBin,
{
    fn de_bin(o: &mut usize, d: &[u8]) -> Result<Self, DeBinErr> {
        let len: usize = DeBin::de_bin(o, d)?;
        let mut h = HashMap::new();
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
