//! UI generator for [serde](serde_core) compatible types

#[repr(transparent)]
struct UiSerializer<'a>(&'a mut egui::Ui);

#[derive(Debug, derive_more::Display, derive_more::Error)]
enum UiSerializerErr {
    #[display("{}", _0)]
    Custom(#[error(not(source))] String),
}

impl serde_core::ser::Error for UiSerializerErr {
    fn custom<T>(msg: T) -> Self
    where
        T: core::fmt::Display,
    {
        Self::Custom(msg.to_string())
    }
}

impl<'a> serde_core::Serializer for UiSerializer<'a> {
    type Ok = ();

    type Error = UiSerializerErr;

    type SerializeSeq = UiSerializerSeq<'a>;

    type SerializeTuple = &'a mut Self;

    type SerializeTupleStruct = &'a mut Self;

    type SerializeTupleVariant = &'a mut Self;

    type SerializeMap = UiSerializerMap<'a>;

    type SerializeStruct = UiSerializerStruct<'a>;

    type SerializeStructVariant = &'a mut Self;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.0.label(v.to_string());

        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.0.label(v.to_string());

        Ok(())
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.0.label(v.to_string());

        Ok(())
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.0.label(v.to_string());

        Ok(())
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.0.label(v.to_string());

        Ok(())
    }

    fn serialize_u8(self, v: u8) -> Result<Self::Ok, Self::Error> {
        self.0.label(v.to_string());

        Ok(())
    }

    fn serialize_u16(self, v: u16) -> Result<Self::Ok, Self::Error> {
        self.0.label(v.to_string());

        Ok(())
    }

    fn serialize_u32(self, v: u32) -> Result<Self::Ok, Self::Error> {
        self.0.label(v.to_string());

        Ok(())
    }

    fn serialize_u64(self, v: u64) -> Result<Self::Ok, Self::Error> {
        self.0.label(v.to_string());

        Ok(())
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.0.label(v.to_string());

        Ok(())
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.0.label(v.to_string());

        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.0.label(format!("{v}"));

        Ok(())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.0.label(v);

        Ok(())
    }

    fn serialize_bytes(self, v: &[u8]) -> Result<Self::Ok, Self::Error> {
        todo!()
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.0.label("");

        Ok(())
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde_core::Serialize,
    {
        value.serialize(UiSerializer(self.0))
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        self.0.label("()");

        Ok(())
    }

    fn serialize_unit_struct(self, name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.0.label(name);

        Ok(())
    }

    fn serialize_unit_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        _ = variant_index;

        self.0.label(format!("{name}::{variant}"));

        Ok(())
    }

    fn serialize_newtype_struct<T>(
        self,
        name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde_core::Serialize,
    {
        todo!()
    }

    fn serialize_newtype_variant<T>(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + serde_core::Serialize,
    {
        todo!()
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        _ = len;

        Ok(UiSerializerSeq(self.0))
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        todo!()
    }

    fn serialize_tuple_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        todo!()
    }

    fn serialize_tuple_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        todo!()
    }

    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        _ = len;

        Ok(UiSerializerMap(self.0))
    }

    fn serialize_struct(
        self,
        name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        _ = len;

        self.0.label(name);

        Ok(UiSerializerStruct(self.0))
    }

    fn serialize_struct_variant(
        self,
        name: &'static str,
        variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        todo!()
    }

    fn serialize_i128(self, v: i128) -> Result<Self::Ok, Self::Error> {
        let _ = v;
        Err(serde_core::ser::Error::custom("i128 is not supported"))
    }

    fn serialize_u128(self, v: u128) -> Result<Self::Ok, Self::Error> {
        let _ = v;
        Err(serde_core::ser::Error::custom("u128 is not supported"))
    }

    fn is_human_readable(&self) -> bool {
        true
    }
}

struct UiSerializerSeq<'a>(&'a mut egui::Ui);

impl<'a> serde_core::ser::SerializeSeq for UiSerializerSeq<'a> {
    type Ok = ();

    type Error = UiSerializerErr;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde_core::Serialize,
    {
        value.serialize(UiSerializer(self.0))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a> serde_core::ser::SerializeTuple for &mut UiSerializer<'a> {
    type Ok = ();

    type Error = UiSerializerErr;

    fn serialize_element<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde_core::Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

impl<'a> serde_core::ser::SerializeTupleStruct for &mut UiSerializer<'a> {
    type Ok = ();

    type Error = UiSerializerErr;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde_core::Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

impl<'a> serde_core::ser::SerializeTupleVariant for &mut UiSerializer<'a> {
    type Ok = ();

    type Error = UiSerializerErr;

    fn serialize_field<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde_core::Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

struct UiSerializerMap<'a>(&'a mut egui::Ui);

impl<'a> serde_core::ser::SerializeMap for UiSerializerMap<'a> {
    type Ok = ();

    type Error = UiSerializerErr;

    fn serialize_key<T>(&mut self, key: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde_core::Serialize,
    {
        key.serialize(UiSerializer(self.0))
    }

    fn serialize_value<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde_core::Serialize,
    {
        value.serialize(UiSerializer(self.0))
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

struct UiSerializerStruct<'a>(&'a mut egui::Ui);

impl<'a> serde_core::ser::SerializeStruct for UiSerializerStruct<'a> {
    type Ok = ();

    type Error = UiSerializerErr;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde_core::Serialize,
    {
        self.0.label(key);
        value.serialize(UiSerializer(self.0))?;

        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(())
    }
}

impl<'a> serde_core::ser::SerializeStructVariant for &mut UiSerializer<'a> {
    type Ok = ();

    type Error = UiSerializerErr;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + serde_core::Serialize,
    {
        todo!()
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        todo!()
    }
}

pub fn generate_ui_for<T: serde_core::Serialize + ?Sized>(value: &T, ui: &mut egui::Ui) {
    let serializer = UiSerializer(ui);

    value.serialize(serializer).unwrap();
}
