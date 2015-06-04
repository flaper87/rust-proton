use std::string;
use std::{char, f64, fmt, io, num, str, mem};
use std::ffi::CString;
use rustc_serialize as serialize;
use std::marker::PhantomData;
use proton_sys;

/// The errors that can arise while parsing a JSON stream.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ErrorCode {
    InvalidSyntax,
    InvalidNumber,
    EOFWhileParsingObject,
    EOFWhileParsingArray,
    EOFWhileParsingValue,
    EOFWhileParsingString,
    KeyMustBeAString,
    ExpectedColon,
    TrailingCharacters,
    TrailingComma,
    InvalidEscape,
    InvalidUnicodeCodePoisize,
    LoneLeadingSurrogateInHexEscape,
    UnexpectedEndOfHexEscape,
    UnrecognizedHex,
    NotFourDigit,
    NotUtf8,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ParserError {
    /// msg, line, col
    SyntaxError(ErrorCode, usize, usize),
    IoError(io::ErrorKind, &'static str),
}

// Builder and Parser have the same errors.
pub type BuilderError = ParserError;

#[derive(Clone, PartialEq, Debug)]
pub enum DecoderError {
    ParseError(ParserError),
    ExpectedError(string::String, string::String),
    MissingFieldError(string::String),
    UnknownVariantError(string::String),
    ApplicationError(string::String)
}


#[derive(Clone, Copy, Debug)]
pub enum EncoderError {
    FmtError(fmt::Error),
    BadHashmapKey,
}

pub type EncodeResult = Result<(), EncoderError>;
pub type DecodeResult<T> = Result<T, DecoderError>;

pub struct Encoder<'e> {
    data: *mut proton_sys::pn_data_t,
    __phantom: PhantomData<&'e ()>
}

impl<'e> Encoder<'e> {
    fn new() -> Encoder<'e> {
        Encoder::with_capacity(16)
    }

    fn with_capacity(capacity: ::libc::size_t) -> Encoder<'e> {
        let data = unsafe{proton_sys::pn_data(capacity)};
        Encoder {data: data, __phantom: PhantomData}
    }
}

impl<'a> serialize::Encoder for Encoder<'a> {
    type Error = EncoderError;

    fn emit_nil(&mut self) -> EncodeResult {
        Ok(unsafe{proton_sys::pn_data_put_null(&mut *self.data);})
    }

    fn emit_usize(&mut self, v: usize) -> EncodeResult {
        Ok(unsafe{proton_sys::pn_data_put_uint(&mut *self.data, v as u32);})
    }
    fn emit_u64(&mut self, v: u64) -> EncodeResult {
        // NOTE(flaper87): check word size
        Ok(unsafe{proton_sys::pn_data_put_ulong(&mut *self.data, v);})
    }
    fn emit_u32(&mut self, v: u32) -> EncodeResult {
        Ok(unsafe{proton_sys::pn_data_put_uint(&mut *self.data, v);})
    }
    fn emit_u16(&mut self, v: u16) -> EncodeResult {
        Ok(unsafe{proton_sys::pn_data_put_ushort(&mut *self.data, v);})
    }
    fn emit_u8(&mut self, v: u8) -> EncodeResult {
        Ok(unsafe{proton_sys::pn_data_put_ubyte(&mut *self.data, v);})
    }

    fn emit_isize(&mut self, v: isize) -> EncodeResult {
        // NOTE(flaper87): check word size
        Ok(unsafe{proton_sys::pn_data_put_int(&mut *self.data, v as i32);})
    }
    fn emit_i64(&mut self, v: i64) -> EncodeResult {
        Ok(unsafe{proton_sys::pn_data_put_long(&mut *self.data, v);})
    }
    fn emit_i32(&mut self, v: i32) -> EncodeResult {
        Ok(unsafe{proton_sys::pn_data_put_int(&mut *self.data, v);})
    }
    fn emit_i16(&mut self, v: i16) -> EncodeResult {
        Ok(unsafe{proton_sys::pn_data_put_short(&mut *self.data, v);})
    }
    fn emit_i8(&mut self, v: i8) -> EncodeResult {
        Ok(unsafe{proton_sys::pn_data_put_byte(&mut *self.data, v);})
    }

    fn emit_bool(&mut self, v: bool) -> EncodeResult {
        Ok(unsafe{proton_sys::pn_data_put_bool(&mut *self.data, v as u8);})
    }

    fn emit_f64(&mut self, v: f64) -> EncodeResult {
        Ok(unsafe{proton_sys::pn_data_put_float(&mut *self.data, v as f32);})
    }
    fn emit_f32(&mut self, v: f32) -> EncodeResult {
        Ok(unsafe{proton_sys::pn_data_put_float(&mut *self.data, v);})
    }

    fn emit_char(&mut self, v: char) -> EncodeResult {
        Ok(unsafe{proton_sys::pn_data_put_char(&mut *self.data, v as u32);})
    }

    fn emit_str(&mut self, slice: &str) -> EncodeResult {
        Ok(unsafe{
            let s = CString::new(slice).unwrap();
            let bytes = proton_sys::pn_bytes(slice.len() as proton_sys::size_t, s.as_ptr());
            proton_sys::pn_data_put_string(&mut *self.data, bytes);})
    }

    fn emit_enum<F>(&mut self, _name: &str, f: F) -> EncodeResult where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult,
    {
        f(self)
    }

    fn emit_enum_variant<F>(&mut self,
                            name: &str,
                            _id: usize,
                            cnt: usize,
                            f: F) -> EncodeResult where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult,
    {
        Ok(())
    }

    fn emit_enum_variant_arg<F>(&mut self, idx: usize, f: F) -> EncodeResult where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult,
    {
        f(self)
    }

    fn emit_enum_struct_variant<F>(&mut self,
                                   name: &str,
                                   id: usize,
                                   cnt: usize,
                                   f: F) -> EncodeResult where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult,
    {
        self.emit_enum_variant(name, id, cnt, f)
    }

    fn emit_enum_struct_variant_field<F>(&mut self,
                                         _: &str,
                                         idx: usize,
                                         f: F) -> EncodeResult where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult,
    {
        self.emit_enum_variant_arg(idx, f)
    }

    fn emit_struct<F>(&mut self, _: &str, _: usize, f: F) -> EncodeResult where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult,
    {
        Ok(())
    }

    fn emit_struct_field<F>(&mut self, name: &str, idx: usize, f: F) -> EncodeResult where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult,
    {
        f(self)
    }

    fn emit_tuple<F>(&mut self, len: usize, f: F) -> EncodeResult where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult,
    {
        self.emit_seq(len, f)
    }
    fn emit_tuple_arg<F>(&mut self, idx: usize, f: F) -> EncodeResult where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult,
    {
        self.emit_seq_elt(idx, f)
    }

    fn emit_tuple_struct<F>(&mut self, _name: &str, len: usize, f: F) -> EncodeResult where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult,
    {
        self.emit_seq(len, f)
    }
    fn emit_tuple_struct_arg<F>(&mut self, idx: usize, f: F) -> EncodeResult where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult,
    {
        self.emit_seq_elt(idx, f)
    }

    fn emit_option<F>(&mut self, f: F) -> EncodeResult where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult,
    {
        f(self)
    }
    fn emit_option_none(&mut self) -> EncodeResult {
        self.emit_nil()
    }
    fn emit_option_some<F>(&mut self, f: F) -> EncodeResult where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult,
    {
        f(self)
    }

    fn emit_seq<F>(&mut self, _len: usize, f: F) -> EncodeResult where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult,
    {
        Ok(())
    }

    fn emit_seq_elt<F>(&mut self, idx: usize, f: F) -> EncodeResult where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult,
    {
        f(self)
    }

    fn emit_map<F>(&mut self, _len: usize, f: F) -> EncodeResult where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult,
    {
        Ok(())
    }

    fn emit_map_elt_key<F>(&mut self, idx: usize, f: F) -> EncodeResult where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult,
    {
        Ok(())
    }

    fn emit_map_elt_val<F>(&mut self, _idx: usize, f: F) -> EncodeResult where
        F: FnOnce(&mut Encoder<'a>) -> EncodeResult,
    {
        f(self)
    }
}

/// Shortcut function to encode a `T` into a JSON `String`
pub fn encode<T: serialize::Encodable>(object: &T) -> Result<Vec<i8>, EncoderError> {
    let mut encoder = Encoder::new();
    try!(object.encode(&mut encoder));
    let mut size = 1024;
    let mut bytes = Vec::with_capacity(size);
    let mut result = proton_sys::PN_OVERFLOW;

    while result == proton_sys::PN_OVERFLOW {
        result = unsafe{proton_sys::pn_data_encode(encoder.data,
                                                   bytes.as_mut_ptr(),
                                                   size as u64) as i8};

        if result > 0 {
            unsafe {bytes.set_len(result as usize);}
            break;
        } else if result <= 0 {
            // not ok
            println!("ERROR");
            break;
        }

        size *= 2;
        bytes = Vec::with_capacity(size);
    }

    bytes.shrink_to_fit();
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use proton_sys;

    #[test]
    fn test_ushort_encoding() {
        let value = "testing";
        let encoded = encode(&value).unwrap();
        let mut data;
        unsafe {
            data = proton_sys::pn_data(1024);
            proton_sys::pn_data_decode(data,
                                       encoded.as_ptr(),
                                       encoded.len() as u64);
        }

        assert_eq!(1, unsafe{proton_sys::pn_data_size(data)});

    }
}