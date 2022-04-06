use std::{borrow::Borrow, cmp::PartialEq, fmt, mem::MaybeUninit};

pub struct ArrayMap<K, V, const SIZE: usize> {
    map: [MaybeUninit<(K, V)>; SIZE],
    len: usize,
}

impl<K, V, const SIZE: usize> ArrayMap<K, V, SIZE> {
    const ELEMENT: MaybeUninit<(K, V)> = MaybeUninit::uninit();

    #[inline]
    pub const fn new() -> Self {
        Self {
            map: [Self::ELEMENT; SIZE],
            len: 0,
        }
    }

    #[inline(always)]
    pub fn capacity(&self) -> usize {
        SIZE
    }

    #[inline(always)]
    pub const fn len(&self) -> usize {
        self.len
    }

    #[inline(always)]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn clear(&mut self) {
        unsafe {
            let len = self.len();

            if 0 < len {
                self.len = 0;

                let tail = std::slice::from_raw_parts_mut(self.map.as_mut_ptr().add(0), len);

                std::ptr::drop_in_place(tail);
            }
        }
    }

    pub fn insert(&mut self, key: K, value: V) -> Option<V>
    where
        K: PartialEq,
    {
        if let Some(entry) = self.get_mut(&key) {
            let mut temp = value;

            std::mem::swap(&mut temp, entry);

            Some(temp)
        } else if self.len < SIZE {
            let len = self.len;

            debug_assert!(len < SIZE);

            unsafe {
                std::ptr::write(
                    self.map.as_mut_ptr().add(len),
                    MaybeUninit::new((key, value)),
                );
            }

            self.len = len + 1;

            None
        } else {
            Some(value)
        }
    }

    #[must_use]
    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&V>
    where
        Q: PartialEq,
        K: Borrow<Q>,
    {
        for item in &self.map[0..self.len] {
            let (k, v) = unsafe { item.assume_init_ref() };

            if key.eq(k.borrow()) {
                return Some(v);
            }
        }

        None
    }

    #[must_use]
    pub fn get_mut<Q>(&mut self, key: Q) -> Option<&mut V>
    where
        Q: Borrow<K>,
        K: PartialEq,
    {
        for item in &mut self.map[0..self.len] {
            let (k, v) = unsafe { item.assume_init_mut() };

            if k == key.borrow() {
                return Some(v);
            }
        }

        None
    }

    pub fn contains<Q: ?Sized>(&self, key: &Q) -> bool
    where
        Q: PartialEq,
        K: Borrow<Q>,
    {
        self.get(key).is_some()
    }

    pub fn iter(&self) -> Iter<'_, K, V, SIZE> {
        Iter {
            map: self,
            index: 0,
        }
    }

    fn as_slice(&self) -> &[(K, V)] {
        let len = self.len();

        unsafe { std::slice::from_raw_parts(self.map.as_ptr() as *const (K, V), len) }
    }
}

impl<K, V, const SIZE: usize> Drop for ArrayMap<K, V, SIZE> {
    fn drop(&mut self) {
        self.clear();
    }
}

impl<'m, K, V, const SIZE: usize> IntoIterator for &'m ArrayMap<K, V, SIZE> {
    type Item = (&'m K, &'m V);

    type IntoIter = Iter<'m, K, V, SIZE>;

    fn into_iter(self) -> Self::IntoIter {
        Iter {
            map: self,
            index: 0,
        }
    }
}

impl<K, V, const SIZE: usize> fmt::Debug for ArrayMap<K, V, SIZE>
where
    K: fmt::Debug,
    V: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.as_slice().iter()).finish()
    }
}

impl<K, V, const SIZE: usize> PartialEq for ArrayMap<K, V, SIZE>
where
    K: PartialEq,
    V: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len && PartialEq::eq(self.as_slice(), other.as_slice())
    }
}

pub struct Iter<'m, K, V, const SIZE: usize> {
    map: &'m ArrayMap<K, V, SIZE>,
    index: usize,
}

impl<'m, K, V, const SIZE: usize> Iterator for Iter<'m, K, V, SIZE> {
    type Item = (&'m K, &'m V);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < SIZE {
            if self.index < self.map.len() {
                let item = &self.map.map[self.index];

                self.index += 1;

                let (k, v) = unsafe { item.assume_init_ref() };

                Some((k, v))
            } else {
                None
            }
        } else {
            None
        }
    }
}
