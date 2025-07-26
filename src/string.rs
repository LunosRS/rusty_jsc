// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::{sys, JSString};
use std::ffi::CString;
use std::fmt;

impl JSString {
    /// Return the number of Unicode characters in this JavaScript string.
    ///
    /// Remember that strings in JavaScript are UTF-16 encoded.
    ///
    /// ```rust
    /// # use javascriptcore::JSString;
    /// let str = JSString::from("😄");
    ///
    /// // The JavaScript string length is 2, since it's UTF-16 encoded.
    /// assert_eq!(str.len(), 2);
    ///
    /// // But once encoded into UTF-8 as a Rust string, it's 4.
    /// assert_eq!(str.to_string().len(), 4);
    /// ```
    pub fn len(&self) -> usize {
        unsafe { sys::JSStringGetLength(self.raw) }
    }

    /// Check whether the string is empty.
    ///
    /// ```rust
    /// # use javascriptcore::JSString;
    /// assert!(JSString::from("").is_empty());
    /// assert!(!JSString::from("abc").is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl fmt::Debug for JSString {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "JSString {{ \"{self}\" }}")
    }
}

impl fmt::Display for JSString {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        // Optimization: Use stack allocation for small strings to avoid heap allocation
        const SMALL_STRING_SIZE: usize = 128;
        
        unsafe {
            let max_size = sys::JSStringGetMaximumUTF8CStringSize(self.raw);
            
            if max_size <= SMALL_STRING_SIZE {
                // For small strings, use stack allocation
                let mut stack_buffer = [0u8; SMALL_STRING_SIZE];
                let actual_size = sys::JSStringGetUTF8CString(
                    self.raw,
                    stack_buffer.as_mut_ptr().cast::<::std::os::raw::c_char>(),
                    SMALL_STRING_SIZE,
                );
                
                // Create a string slice directly from the stack buffer
                // Subtract 1 to remove null terminator
                let s = std::str::from_utf8(&stack_buffer[0..actual_size - 1]).unwrap();
                write!(fmt, "{s}")
            } else {
                // For larger strings, fall back to heap allocation
                let mut buffer: Vec<u8> = Vec::with_capacity(max_size);
                let actual_size = sys::JSStringGetUTF8CString(
                    self.raw,
                    buffer.as_mut_ptr().cast::<::std::os::raw::c_char>(),
                    max_size,
                );
                buffer.set_len(actual_size - 1);
                let s = String::from_utf8(buffer).unwrap();
                write!(fmt, "{s}")
            }
        }
    }
}

impl Drop for JSString {
    fn drop(&mut self) {
        unsafe { sys::JSStringRelease(self.raw) }
    }
}

impl PartialEq for JSString {
    fn eq(&self, other: &JSString) -> bool {
        unsafe { sys::JSStringIsEqual(self.raw, other.raw) }
    }
}

fn js_string_equals_str(js_string: &JSString, rust_str: &str) -> bool {
    // Optimization: Use a stack-allocated buffer for small strings to avoid heap allocation
    const SMALL_STRING_SIZE: usize = 128;
    
    if rust_str.len() < SMALL_STRING_SIZE {
        // For small strings, use a stack-allocated buffer with a null terminator
        let mut buffer = [0u8; SMALL_STRING_SIZE + 1]; // +1 for null terminator
        let bytes = rust_str.as_bytes();
        buffer[..bytes.len()].copy_from_slice(bytes);
        buffer[bytes.len()] = 0; // Null terminator
        
        unsafe { 
            sys::JSStringIsEqualToUTF8CString(js_string.raw, buffer.as_ptr() as *const ::std::os::raw::c_char) 
        }
    } else {
        // For larger strings, fall back to CString
        let utf8 = CString::new(rust_str.as_bytes()).unwrap();
        unsafe { sys::JSStringIsEqualToUTF8CString(js_string.raw, utf8.as_ptr()) }
    }
}

impl<'s> PartialEq<&'s str> for JSString {
    fn eq(&self, other: &&'s str) -> bool {
        js_string_equals_str(self, other)
    }
}

impl PartialEq<String> for JSString {
    fn eq(&self, other: &String) -> bool {
        js_string_equals_str(self, other.as_str())
    }
}

impl PartialEq<JSString> for &str {
    fn eq(&self, other: &JSString) -> bool {
        js_string_equals_str(other, self)
    }
}

impl PartialEq<JSString> for String {
    fn eq(&self, other: &JSString) -> bool {
        js_string_equals_str(other, self.as_str())
    }
}

// Helper function to create a JSString from a Rust string
fn js_string_from_str(s: &str) -> JSString {
    // Optimization: Use a stack-allocated buffer for small strings to avoid heap allocation
    const SMALL_STRING_SIZE: usize = 128;
    
    if s.len() < SMALL_STRING_SIZE {
        // For small strings, use a stack-allocated buffer with a null terminator
        let mut buffer = [0u8; SMALL_STRING_SIZE + 1]; // +1 for null terminator
        let bytes = s.as_bytes();
        buffer[..bytes.len()].copy_from_slice(bytes);
        buffer[bytes.len()] = 0; // Null terminator
        
        JSString {
            raw: unsafe { 
                sys::JSStringCreateWithUTF8CString(buffer.as_ptr() as *const ::std::os::raw::c_char) 
            },
        }
    } else {
        // For larger strings, fall back to CString
        let c = CString::new(s.as_bytes()).unwrap();
        JSString {
            raw: unsafe { sys::JSStringCreateWithUTF8CString(c.as_ptr()) },
        }
    }
}

impl From<&str> for JSString {
    fn from(s: &str) -> Self {
        js_string_from_str(s)
    }
}

impl From<String> for JSString {
    fn from(s: String) -> Self {
        js_string_from_str(&s)
    }
}

impl<'s> From<&'s JSString> for String {
    fn from(s: &'s JSString) -> Self {
        s.to_string()
    }
}

#[cfg(test)]
mod tests {
    use crate::JSString;

    #[test]
    fn from_conversion() {
        let a: JSString = "abc".into();
        let b: JSString = "abc".to_owned().into();
        assert_eq!(a, a);
        assert_eq!(a, b);
        assert_eq!(b, b);

        let c: JSString = "def".into();
        assert_ne!(a, c);

        let d: JSString = "abcdef".into();
        assert_ne!(a, d);

        let e: String = (&d).into();
        assert_eq!(e, "abcdef");
    }

    #[test]
    fn equality() {
        let a: JSString = "abc".into();
        let s: String = "abc".to_owned();

        assert_eq!(a, "abc");
        assert_eq!(a, s);

        assert_eq!("abc", a);
        assert_eq!(s, a);
    }

    #[test]
    fn len() {
        let a: JSString = "😄".into();

        assert_eq!(a.len(), 2);
        assert_eq!(a.to_string().len(), 4);

        let b: JSString = "∀𝑥∈ℝ,𝑥²≥0".into();

        assert_eq!(b.len(), 11);
        assert_eq!(b.to_string().len(), 24);
    }

    #[test]
    fn is_empty() {
        assert!(JSString::from("").is_empty());
        assert!(!JSString::from("abc").is_empty());
    }
}
