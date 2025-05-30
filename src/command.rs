#[cfg(not(feature = "command"))]
pub mod missing_feature {
    pub const ERR: &'static str = "missing `command` cargo feature";

    pub fn serialize<S: serde::Serializer>(_: S) -> Result<S::Ok, S::Error> {
        Err(serde::ser::Error::custom(ERR))
    }

    pub fn deserialize<'de, D: serde::Deserializer<'de>>(_: D) -> Result<(), D::Error> {
        Err(serde::de::Error::custom(ERR))
    }
}
