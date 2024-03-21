use std::io;

use serde::Serialize;

use crate::error::{Error, ErrorCode, Result};

// TODO: Make configurable
const INDENT: [u8; 2] = [0x20, 0x20];

/// A container for serializing Rust values into SJSON.
pub struct Serializer<W> {
    // The current indentation level
    level: usize,
    writer: W,
}

/// Serializes a value into a generic `io::Write`.
#[inline]
pub fn to_writer<T, W>(writer: &mut W, value: &T) -> Result<()>
where
    W: io::Write,
    T: Serialize,
{
    let mut serializer = Serializer::new(writer);
    value.serialize(&mut serializer)
}

/// Serializes a value into a byte vector.
#[inline]
pub fn to_vec<T>(value: &T) -> Result<Vec<u8>>
where
    T: Serialize,
{
    let mut vec = Vec::with_capacity(128);
    to_writer(&mut vec, value)?;
    Ok(vec)
}

/// Serializes a value into a string.
#[inline]
pub fn to_string<T>(value: &T) -> Result<String>
where
    T: Serialize,
{
    let vec = to_vec(value)?;
    let string = if cfg!(debug_assertions) {
        String::from_utf8(vec).expect("We do not emit invalid UTF-8")
    } else {
        unsafe { String::from_utf8_unchecked(vec) }
    };
    Ok(string)
}

impl<W> Serializer<W>
where
    W: io::Write,
{
    /// Creates a new `Serializer`.
    pub fn new(writer: W) -> Self {
        Self { level: 0, writer }
    }

    #[inline]
    fn write(&mut self, bytes: impl AsRef<[u8]>) -> Result<()> {
        self.writer.write_all(bytes.as_ref()).map_err(Error::from)
    }

    #[inline]
    fn add_indent(&mut self) -> Result<()> {
        for _ in 0..self.level.saturating_sub(1) {
            self.write(INDENT)?;
        }

        Ok(())
    }

    #[inline]
    fn ensure_top_level_struct(&self) -> Result<()> {
        if self.level == 0 {
            return Err(Error::new(ErrorCode::ExpectedTopLevelObject, 0, 0, None));
        }

        Ok(())
    }
}

impl<'a, W> serde::ser::Serializer for &'a mut Serializer<W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Self;
    type SerializeTuple = Self;
    type SerializeTupleStruct = Self;
    type SerializeTupleVariant = Self;
    type SerializeMap = Self;
    type SerializeStruct = Self;
    type SerializeStructVariant = Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok> {
        self.ensure_top_level_struct()?;
        self.write(if v { "true" } else { "false" })?;
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok> {
        self.serialize_i64(v.into())
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok> {
        self.serialize_i64(v.into())
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok> {
        self.serialize_i64(v.into())
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok> {
        self.ensure_top_level_struct()?;
        self.serialize_str(&format!("{}", v))
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok> {
        self.serialize_u64(v.into())
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok> {
        self.serialize_u64(v.into())
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok> {
        self.serialize_u64(v.into())
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok> {
        self.ensure_top_level_struct()?;
        self.serialize_str(&format!("{}", v))
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok> {
        self.serialize_f64(v.into())
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok> {
        self.ensure_top_level_struct()?;
        if !v.is_finite() {
            return Err(Error::new(ErrorCode::NonFiniteFloat, 0, 0, None));
        }

        self.serialize_str(&format!("{}", v))
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok> {
        let mut buf = [0; 4];
        self.serialize_bytes(v.encode_utf8(&mut buf).as_bytes())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        self.ensure_top_level_struct()?;

        let needs_quotes =
            v.is_empty() || v.contains([' ', '\n', '\r', '\t', '=', '\'', '"', '\\', ':']);

        if needs_quotes {
            self.write(b"\"")?;

            // Since we've added a layer of quotes, we now need to escape
            // certain characters.
            for c in v.chars() {
                match c {
                    '\t' => {
                        self.write(b"\\")?;
                        self.write(b"t")?;
                    }
                    '\n' => {
                        self.write(b"\\")?;
                        self.write(b"n")?;
                    }
                    '\r' => {
                        self.write(b"\\")?;
                        self.write(b"r")?;
                    }
                    '"' => {
                        self.write(b"\\")?;
                        self.write(b"\"")?;
                    }
                    '\\' => {
                        self.write(b"\\")?;
                        self.write(b"\\")?;
                    }
                    c => {
                        self.serialize_char(c)?;
                    }
                };
            }

            self.write(b"\"")?;
        } else {
            self.write(v.as_bytes())?;
        }

        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok> {
        self.ensure_top_level_struct()?;
        // For now we assume that the byte array contains
        // valid SJSON.
        // TODO: Turn this into an actual array of encoded bytes.
        self.write(v)
    }

    fn serialize_none(self) -> Result<Self::Ok> {
        self.serialize_unit()
    }

    fn serialize_some<T: ?Sized>(self, value: &T) -> Result<Self::Ok>
    where
        T: serde::Serialize,
    {
        self.ensure_top_level_struct()?;

        // A present value is represented as just that value.
        // Just like JSON, we do not distinguish between `None` and `Some(())`.
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok> {
        self.ensure_top_level_struct()?;
        self.write(b"null")
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok> {
        self.serialize_str(variant)
    }

    fn serialize_newtype_struct<T: ?Sized>(self, _name: &'static str, value: &T) -> Result<Self::Ok>
    where
        T: serde::Serialize,
    {
        self.ensure_top_level_struct()?;

        value.serialize(self)
    }

    // Serialize an externally tagged enum: `{ NAME = VALUE }`.
    fn serialize_newtype_variant<T: ?Sized>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok>
    where
        T: serde::Serialize,
    {
        self.ensure_top_level_struct()?;

        self.write(b"{ ")?;
        variant.serialize(&mut *self)?;
        self.write(b" = ")?;
        value.serialize(&mut *self)?;
        self.write(b" }")
    }

    // Serialize the start of a sequence.
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        self.ensure_top_level_struct()?;

        self.write(b"[\n")?;
        self.level += 1;
        Ok(self)
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        // SJSON, like JSON, does not distinguish a tuple from an array.
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        // A tuple struct also serializes into an array.
        self.serialize_seq(Some(len))
    }

    // Serialize the externally tagged representation of tuple structs: `{ NAME = [DATA...] }`.
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        self.ensure_top_level_struct()?;

        variant.serialize(&mut *self)?;

        self.write(b" = [\n")?;
        self.level += 1;

        Ok(self)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        if self.level > 0 {
            self.write(b"{\n")?;
        }
        self.level += 1;
        Ok(self)
    }

    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.serialize_map(Some(len))
    }

    // Serialize the externally tagged representation: `{ NAME = { K = V, ... } }`.
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        self.ensure_top_level_struct()?;

        variant.serialize(&mut *self)?;

        self.write(b" = {\n")?;
        self.level += 1;

        Ok(self)
    }

    fn collect_str<T: ?Sized>(self, value: &T) -> Result<Self::Ok>
    where
        T: std::fmt::Display,
    {
        self.serialize_str(&value.to_string())
    }
}

impl<'a, W> serde::ser::SerializeSeq for &'a mut Serializer<W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.add_indent()?;
        value.serialize(&mut **self)?;
        self.write(b"\n")
    }

    fn end(self) -> Result<Self::Ok> {
        self.level -= 1;
        self.add_indent()?;
        self.write(b"]")
    }
}

impl<'a, W> serde::ser::SerializeTuple for &'a mut Serializer<W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.add_indent()?;
        value.serialize(&mut **self)?;
        self.write(b"\n")
    }

    fn end(self) -> Result<Self::Ok> {
        self.level -= 1;
        self.add_indent()?;
        self.write(b"]")
    }
}

impl<'a, W> serde::ser::SerializeTupleStruct for &'a mut Serializer<W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.add_indent()?;
        value.serialize(&mut **self)?;
        self.write(b"\n")
    }

    fn end(self) -> Result<Self::Ok> {
        self.level -= 1;
        self.add_indent()?;
        self.write(b"]")
    }
}

impl<'a, W> serde::ser::SerializeTupleVariant for &'a mut Serializer<W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.add_indent()?;
        value.serialize(&mut **self)?;
        self.write(b"\n")
    }

    fn end(self) -> Result<Self::Ok> {
        self.level -= 1;
        self.add_indent()?;
        self.write(b"]\n")?;

        self.level -= 1;

        if self.level > 0 {
            self.add_indent()?;
            self.write(b"}")?;
        }

        Ok(())
    }
}

impl<'a, W> serde::ser::SerializeMap for &'a mut Serializer<W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.add_indent()?;
        key.serialize(&mut **self)
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        // It doesn't make a difference where the `=` is added. But doing it here
        // means `serialize_key` is only a call to a different function, which should
        // have greater optimization potential for the compiler.
        self.write(b" = ")?;
        value.serialize(&mut **self)?;
        self.write(b"\n")
    }

    fn end(self) -> Result<Self::Ok> {
        if self.level > 1 {
            self.level -= 1;
            self.add_indent()?;
            self.write(b"}")?;
        }
        Ok(())
    }
}

impl<'a, W> serde::ser::SerializeStruct for &'a mut Serializer<W>
where
    W: io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.add_indent()?;
        key.serialize(&mut **self)?;

        self.write(b" = ")?;

        value.serialize(&mut **self)?;
        self.write(b"\n")
    }

    fn end(self) -> Result<Self::Ok> {
        if self.level > 1 {
            self.level -= 1;
            self.add_indent()?;
            self.write(b"}")?;
        }
        Ok(())
    }
}

impl<'a, W> serde::ser::SerializeStructVariant for &'a mut Serializer<W>
where
    W: std::io::Write,
{
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.add_indent()?;
        key.serialize(&mut **self)?;
        self.write(b" = ")?;
        value.serialize(&mut **self)?;
        self.write(b"\n")
    }

    fn end(self) -> Result<Self::Ok> {
        if self.level > 0 {
            self.level -= 1;
            self.add_indent()?;
            self.write(b"}")?;
        }
        Ok(())
    }
}
