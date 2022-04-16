use std::{
    collections::{BTreeMap, BTreeSet, HashMap, HashSet},
    net::{Ipv4Addr, SocketAddr, SocketAddrV4, SocketAddrV6},
    path::{Path, PathBuf},
};

use crate::{uDebug, uDisplay, uWrite, Formatter};

impl<T> uDebug for Box<T>
where
    T: uDebug,
{
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        <T as uDebug>::fmt(self, f)
    }
}

impl<T> uDisplay for Box<T>
where
    T: uDisplay,
{
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        <T as uDisplay>::fmt(self, f)
    }
}

impl<K, V> uDebug for BTreeMap<K, V>
where
    K: uDebug,
    V: uDebug,
{
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        f.debug_map()?.entries(self)?.finish()
    }
}

impl<T> uDebug for BTreeSet<T>
where
    T: uDebug,
{
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        f.debug_set()?.entries(self)?.finish()
    }
}

impl<K, V> uDebug for HashMap<K, V>
where
    K: uDebug,
    V: uDebug,
{
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        f.debug_map()?.entries(self)?.finish()
    }
}

impl<T> uDebug for HashSet<T>
where
    T: uDebug,
{
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        f.debug_set()?.entries(self)?.finish()
    }
}

// TODO
// impl uDebug for String {
//     fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
//     where
//         W: uWrite + ?Sized,
//     {
//         <str as uDebug>::fmt(self, f)
//     }
// }

impl uDisplay for String {
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        <str as uDisplay>::fmt(self, f)
    }
}

impl<T> uDebug for Vec<T>
where
    T: uDebug,
{
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        <[T] as uDebug>::fmt(self, f)
    }
}

impl uDisplay for Path {
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        <str as uDisplay>::fmt(self.as_os_str().to_string_lossy().as_ref(), f)
    }
}

impl uDisplay for PathBuf {
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        <Path as uDisplay>::fmt(self.as_path(), f)
    }
}

impl uDisplay for SocketAddr {
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        match self {
            SocketAddr::V4(v4) => uDisplay::fmt(v4, f),
            SocketAddr::V6(v6) => uDisplay::fmt(v6, f),
        }
    }
}

impl uDisplay for SocketAddrV4 {
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        use crate as vfmt;

        vfmt::uwrite!(f, "{}:{}", self.ip(), self.port())
    }
}

impl uDisplay for Ipv4Addr {
    fn fmt<W>(&self, f: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        use crate as vfmt;

        let octets = self.octets();

        vfmt::uwrite!(f, "{}.{}.{}.{}", octets[0], octets[1], octets[2], octets[3])
    }
}

impl uDisplay for SocketAddrV6 {
    fn fmt<W>(&self, _: &mut Formatter<'_, W>) -> Result<(), W::Error>
    where
        W: uWrite + ?Sized,
    {
        Ok(())
    }
}
