use std::fmt;
use std::marker::PhantomData;
use std::str::FromStr;
use std::time::Duration;

use serde::{Deserialize, Deserializer, de};

use super::{Records, ZoneRecord};

pub(super) fn default_interval() -> Duration {
    Duration::from_secs(300)
}

pub(super) fn duration_from_secs<'de, D: Deserializer<'de>>(d: D) -> Result<Duration, D::Error> {
    Ok(Duration::from_secs(u64::deserialize(d)?))
}

/// Deserializes a value that can be expressed as either a string or a map.
///
/// The type `T` must implement [`Deserialize`] for the map form and [`FromStr`]
/// for the string form. Mirrors the pattern from the serde documentation.
pub(super) fn string_or_struct<'de, T, D>(d: D) -> Result<T, D::Error>
where
    T: Deserialize<'de> + FromStr,
    <T as FromStr>::Err: fmt::Display,
    D: Deserializer<'de>,
{
    struct StringOrStruct<T>(PhantomData<fn() -> T>);

    impl<'de, T> de::Visitor<'de> for StringOrStruct<T>
    where
        T: Deserialize<'de> + FromStr,
        <T as FromStr>::Err: fmt::Display,
    {
        type Value = T;

        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("string or map")
        }

        fn visit_str<E: de::Error>(self, v: &str) -> Result<T, E> {
            v.parse().map_err(de::Error::custom)
        }

        fn visit_map<M: de::MapAccess<'de>>(self, map: M) -> Result<T, M::Error> {
            T::deserialize(de::value::MapAccessDeserializer::new(map))
        }
    }

    d.deserialize_any(StringOrStruct(PhantomData))
}

pub(super) fn deserialize_records<'de, D: Deserializer<'de>>(d: D) -> Result<Records, D::Error> {
    #[derive(Deserialize)]
    struct RecordEntry {
        name: String,
        zone: String,
        #[serde(default)]
        v4: bool,
        #[serde(default)]
        v6: bool,
    }

    let entries = Vec::<RecordEntry>::deserialize(d)?;
    let mut records = Records::default();
    for rec in entries {
        let zone_record = ZoneRecord {
            zone: rec.zone,
            name: rec.name,
        };
        if rec.v4 {
            records.v4.push(zone_record.clone());
        }
        if rec.v6 {
            records.v6.push(zone_record);
        }
    }
    Ok(records)
}
