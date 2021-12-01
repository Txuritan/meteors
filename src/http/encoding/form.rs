// Copyright 2013-2016 The rust-url developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Parser and serializer for the [`application/x-www-form-urlencoded` syntax](
//! http://url.spec.whatwg.org/#application/x-www-form-urlencoded),
//! as used by HTML forms.
//!
//! Converts between a string (such as an URL’s query string)
//! and a sequence of (name, value) pairs.

use std::{
    borrow::Cow,
    str,
};

use crate::http::encoding::percent::{percent_decode, percent_encode_byte, decode_utf8_lossy};

/// Convert a byte string in the `application/x-www-form-urlencoded` syntax
/// into a iterator of (name, value) pairs.
///
/// Use `parse(input.as_bytes())` to parse a `&str` string.
///
/// The names and values are percent-decoded. For instance, `%23first=%25try%25` will be
/// converted to `[("#first", "%try%")]`.
#[inline]
pub fn parse(input: &[u8]) -> Parse<'_> {
    Parse { input }
}
/// The return type of `parse()`.
#[derive(Copy, Clone)]
pub struct Parse<'a> {
    input: &'a [u8],
}

impl<'a> Iterator for Parse<'a> {
    type Item = (Cow<'a, str>, Cow<'a, str>);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.input.is_empty() {
                return None;
            }
            let mut split2 = self.input.splitn(2, |&b| b == b'&');
            let sequence = split2.next().unwrap();
            self.input = split2.next().unwrap_or(&[][..]);
            if sequence.is_empty() {
                continue;
            }
            let mut split2 = sequence.splitn(2, |&b| b == b'=');
            let name = split2.next().unwrap();
            let value = split2.next().unwrap_or(&[][..]);
            return Some((decode(name), decode(value)));
        }
    }
}

fn decode(input: &[u8]) -> Cow<'_, str> {
    let replaced = replace_plus(input);
    decode_utf8_lossy(match percent_decode(&replaced).into() {
        Cow::Owned(vec) => Cow::Owned(vec),
        Cow::Borrowed(_) => replaced,
    })
}

/// Replace b'+' with b' '
fn replace_plus(input: &[u8]) -> Cow<'_, [u8]> {
    match input.iter().position(|&b| b == b'+') {
        None => Cow::Borrowed(input),
        Some(first_position) => {
            let mut replaced = input.to_owned();
            replaced[first_position] = b' ';
            for byte in &mut replaced[first_position + 1..] {
                if *byte == b'+' {
                    *byte = b' ';
                }
            }
            Cow::Owned(replaced)
        }
    }
}

impl<'a> Parse<'a> {
    /// Return a new iterator that yields pairs of `String` instead of pairs of `Cow<str>`.
    pub fn into_owned(self) -> ParseIntoOwned<'a> {
        ParseIntoOwned { inner: self }
    }
}

/// Like `Parse`, but yields pairs of `String` instead of pairs of `Cow<str>`.
pub struct ParseIntoOwned<'a> {
    inner: Parse<'a>,
}

impl<'a> Iterator for ParseIntoOwned<'a> {
    type Item = (String, String);

    fn next(&mut self) -> Option<Self::Item> {
        self.inner
            .next()
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
    }
}

/// The [`application/x-www-form-urlencoded` byte serializer](
/// https://url.spec.whatwg.org/#concept-urlencoded-byte-serializer).
///
/// Return an iterator of `&str` slices.
pub fn byte_serialize(input: &[u8]) -> ByteSerialize<'_> {
    ByteSerialize { bytes: input }
}

/// Return value of `byte_serialize()`.
#[derive(Debug)]
pub struct ByteSerialize<'a> {
    bytes: &'a [u8],
}

fn byte_serialized_unchanged(byte: u8) -> bool {
    matches!(byte, b'*' | b'-' | b'.' | b'0' ..= b'9' | b'A' ..= b'Z' | b'_' | b'a' ..= b'z')
}

impl<'a> Iterator for ByteSerialize<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<&'a str> {
        if let Some((&first, tail)) = self.bytes.split_first() {
            if !byte_serialized_unchanged(first) {
                self.bytes = tail;
                return Some(if first == b' ' {
                    "+"
                } else {
                    percent_encode_byte(first)
                });
            }
            let position = tail.iter().position(|&b| !byte_serialized_unchanged(b));
            let (unchanged_slice, remaining) = match position {
                // 1 for first_byte + i unchanged in tail
                Some(i) => self.bytes.split_at(1 + i),
                None => (self.bytes, &[][..]),
            };
            self.bytes = remaining;
            // This unsafe is appropriate because we have already checked these
            // bytes in byte_serialized_unchanged, which checks for a subset
            // of UTF-8. So we know these bytes are valid UTF-8, and doing
            // another UTF-8 check would be wasteful.
            Some(unsafe { str::from_utf8_unchecked(unchanged_slice) })
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        if self.bytes.is_empty() {
            (0, Some(0))
        } else {
            (1, Some(self.bytes.len()))
        }
    }
}
