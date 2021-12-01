pub(crate) mod signal;

pub(crate) mod array_map;
pub(crate) mod path_tree;
pub(crate) mod string;
pub(crate) mod thread_pool;

pub(crate) use self::{array_map::ArrayMap, path_tree::PathTree, string::StringExt};

use std::{ops::Range, ptr};

pub(crate) struct Ascii;

impl Ascii {
    const MAX_ITERATIONS: usize = 255;

    pub fn find_index(buf: &[u8], pat: u8) -> Option<usize> {
        let mut i = 0;

        while let Some(byte) = buf.get(i) {
            if *byte == pat {
                break;
            } else if i == Self::MAX_ITERATIONS {
                return None;
            } else {
                i += 1;
            }
        }

        if i == 0 {
            None
        } else {
            Some(i)
        }
    }

    pub fn read_until(buf: &[u8], offset: &mut usize, pat: u8) -> Option<String> {
        let mut i = *offset;

        while let Some(byte) = buf.get(i) {
            if *byte == pat {
                break;
            } else if i == Self::MAX_ITERATIONS {
                return None;
            } else {
                i += 1;
            }
        }

        if i == 0 {
            None
        } else {
            let text = String::from_utf8((&buf[(*offset)..i]).to_vec()).ok()?;

            *offset = i;

            Some(text)
        }
    }
}

pub(crate) struct Const;

#[allow(dead_code)]
impl Const {
    #[inline]
    #[track_caller]
    pub const unsafe fn index_get_unchecked<T>(slice: &[T], index: usize) -> &T {
        debug_assert!(index < slice.len());

        &*(slice.as_ptr().add(index))
    }

    #[inline]
    #[track_caller]
    pub const unsafe fn range_get_unchecked<T>(slice: &[T], range: Range<usize>) -> &[T] {
        debug_assert!(range.start <= range.end);

        let ptr = slice.as_ptr().add(range.start);
        let len = range.end - range.start;

        &*(ptr::slice_from_raw_parts(ptr, len))
    }

    #[inline]
    #[track_caller]
    pub const unsafe fn slice_as_array<T, const LEN: usize>(
        slice: &[T],
        offset: usize,
    ) -> &[T; LEN] {
        debug_assert!(slice.len() > offset + LEN);

        let slice = Self::range_get_unchecked(slice, offset..offset + LEN);

        &*(slice.as_ptr() as *const [_; LEN])
    }
}
