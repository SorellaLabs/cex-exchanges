use serde::Deserialize;

pub trait TryDeserialize: for<'de> Deserialize<'de> {
    fn try_deserialize(value: &serde_json::Value) -> eyre::Result<Option<Self>> {
        Ok(serde_json::from_value::<Self>(value.clone()).ok())
    }
}
