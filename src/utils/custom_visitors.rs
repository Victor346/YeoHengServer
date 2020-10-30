use serde::de::{self, Visitor};
use std::fmt;
use bson::oid::ObjectId;

pub struct ObjectIdVisitor;

impl<'de> Visitor<'de> for ObjectIdVisitor {
    type Value = ObjectId;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("Excepting a string or Objectid")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
    {
        match ObjectId::with_string(v) {
            Ok(oi) => Ok(oi),
            Err(_) => Err(E::custom("Not an ObjectId format")),
        }
    }

    fn visit_map<A>(self, _map: A) -> Result<Self::Value, A::Error>
        where
            A: de::MapAccess<'de>,
    {
        Ok(ObjectId::default())
    }
}