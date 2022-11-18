use serde::Serialize;

use crate::error::{Error, ErrorCode, Result};

// TODO: Make configurable
const INDENT: &str = "  ";

pub struct Serializer {
    // The current indentation level
    level: usize,
    // The output string
    output: String,
}

pub fn to_string<T>(value: &T) -> Result<String>
where
    T: Serialize,
{
    let mut serializer = Serializer {
        level: 0,
        output: String::new(),
    };
    value.serialize(&mut serializer)?;
    Ok(serializer.output)
}

impl Serializer {
    fn add_indent(&mut self) {
        for _ in 0..self.level.saturating_sub(1) {
            self.output += INDENT;
        }
    }

    fn ensure_top_level_struct(&self) -> Result<()> {
        if self.level == 0 {
            return Err(Error::new(ErrorCode::ExpectedTopLevelObject, 0, 0));
        }

        Ok(())
    }
}

impl<'a> serde::ser::Serializer for &'a mut Serializer {
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
        self.output += if v { "true" } else { "false" };
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
        self.output += &v.to_string();
        Ok(())
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
        self.output += &v.to_string();
        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok> {
        if v.is_finite() {
            self.serialize_f64(v.into())
        } else {
            self.ensure_top_level_struct()?;
            self.output += "null";
            Ok(())
        }
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok> {
        self.ensure_top_level_struct()?;
        if v.is_finite() {
            self.output += &v.to_string();
        } else {
            self.output += "null";
        }
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok> {
        let mut buf = [0; 4];
        self.serialize_str(v.encode_utf8(&mut buf))
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok> {
        self.ensure_top_level_struct()?;
        let needs_escapes =
            v.is_empty() || v.contains([' ', '\n', '\r', '\t', '=', '\'', '"', '\\', '/']);
        if needs_escapes {
            self.output += "\"";

            let len = v.len();
            let chars = v.chars();
            let mut start = 0;

            for (i, c) in chars.enumerate() {
                if ('\x20'..='\x7e').contains(&c)
                    && !['\t', '\n', '\r', '\"', '\\', '/'].contains(&c)
                {
                    continue;
                }

                self.output += &v[start..i];
                self.output.push('\\');

                match c {
                    '\t' => {
                        self.output.push('t');
                    }
                    '\n' => {
                        self.output.push('n');
                    }
                    '\r' => {
                        self.output.push('r');
                    }
                    '\x7f'.. => {
                        self.output += &format!("u{:4x}", c as u32);
                    }
                    c => {
                        self.output.push(c);
                    }
                };

                start = i + 1;
            }

            if start < len {
                self.output += &v[start..];
            }

            self.output += "\"";
        } else {
            self.output += v;
        }
        Ok(())
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok> {
        todo!()
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
        self.output += "null";
        Ok(())
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

        self.output += "{ ";
        variant.serialize(&mut *self)?;
        self.output += " = ";
        value.serialize(&mut *self)?;
        self.output += " }\n";
        Ok(())
    }

    // Serialize the start of a sequence.
    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        self.ensure_top_level_struct()?;

        self.output += "[\n";
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

        self.output += " = [\n";
        self.level += 1;

        Ok(self)
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        if self.level > 0 {
            self.output += "{\n";
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

        self.output += " = {\n";
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

impl<'a> serde::ser::SerializeSeq for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.add_indent();
        value.serialize(&mut **self)?;
        if !self.output.ends_with('\n') {
            self.output += "\n";
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        self.level -= 1;
        self.add_indent();
        self.output += "]\n";
        Ok(())
    }
}

impl<'a> serde::ser::SerializeTuple for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_element<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.add_indent();
        value.serialize(&mut **self)?;
        if !self.output.ends_with('\n') {
            self.output += "\n";
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        self.level -= 1;
        self.add_indent();
        self.output += "]\n";
        Ok(())
    }
}

impl<'a> serde::ser::SerializeTupleStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.add_indent();
        value.serialize(&mut **self)?;
        if !self.output.ends_with('\n') {
            self.output += "\n";
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        self.level -= 1;
        self.add_indent();
        self.output += "]\n";
        Ok(())
    }
}

impl<'a> serde::ser::SerializeTupleVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.add_indent();
        value.serialize(&mut **self)?;
        if !self.output.ends_with('\n') {
            self.output += "\n";
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        self.level -= 1;
        self.add_indent();
        self.output += "]\n";
        self.level -= 1;
        if self.level > 0 {
            self.add_indent();
            self.output += "}\n";
        }
        Ok(())
    }
}

impl<'a> serde::ser::SerializeMap for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_key<T: ?Sized>(&mut self, key: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.add_indent();
        key.serialize(&mut **self)
    }

    fn serialize_value<T: ?Sized>(&mut self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        // It doesn't make a difference where the `=` is added. But doing it here
        // means `serialize_key` is only a call to a different function, which should
        // have greater optimization potential for the compiler.
        self.output += " = ";
        value.serialize(&mut **self)?;
        if !self.output.ends_with('\n') {
            self.output += "\n";
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        if self.level > 1 {
            self.level -= 1;
            self.add_indent();
            self.output += "}\n";
        }
        Ok(())
    }
}

impl<'a> serde::ser::SerializeStruct for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.add_indent();
        key.serialize(&mut **self)?;

        self.output += " = ";

        value.serialize(&mut **self)?;
        if !self.output.ends_with('\n') {
            self.output += "\n";
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        if self.level > 1 {
            self.level -= 1;
            self.add_indent();
            self.output += "}\n";
        }
        Ok(())
    }
}

impl<'a> serde::ser::SerializeStructVariant for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    fn serialize_field<T: ?Sized>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        self.add_indent();
        key.serialize(&mut **self)?;
        self.output += " = ";
        value.serialize(&mut **self)?;
        if !self.output.ends_with('\n') {
            self.output += "\n";
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok> {
        if self.level > 0 {
            self.level -= 1;
            self.add_indent();
            self.output += "}\n";
        }
        Ok(())
    }
}
