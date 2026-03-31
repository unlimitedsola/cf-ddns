use std::fmt;
use std::marker::PhantomData;
use std::str::FromStr;
use std::time::Duration;

use serde::{Deserialize, Deserializer, de};

use super::{ProviderConfig, Records, ZoneRecord};
pub(super) const fn default_interval() -> Duration {
    Duration::from_mins(5)
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
    #[derive(Debug, Clone, Eq, PartialEq, Default)]
    enum RecordLookup {
        #[default]
        Disabled,
        Global,
        Custom(ProviderConfig),
    }

    fn bool_or_protocol<'de, D: Deserializer<'de>>(d: D) -> Result<RecordLookup, D::Error> {
        struct BoolOrProtocol;

        impl<'de> de::Visitor<'de> for BoolOrProtocol {
            type Value = RecordLookup;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("bool or record lookup config table")
            }

            fn visit_bool<E: de::Error>(self, v: bool) -> Result<Self::Value, E> {
                Ok(if v {
                    RecordLookup::Global
                } else {
                    RecordLookup::Disabled
                })
            }

            fn visit_map<M: de::MapAccess<'de>>(self, map: M) -> Result<Self::Value, M::Error> {
                #[derive(Deserialize)]
                struct RecordProtocolConfig {
                    #[serde(deserialize_with = "string_or_struct")]
                    lookup: ProviderConfig,
                }
                let config =
                    RecordProtocolConfig::deserialize(de::value::MapAccessDeserializer::new(map))?;
                Ok(RecordLookup::Custom(config.lookup))
            }
        }

        d.deserialize_any(BoolOrProtocol)
    }

    #[derive(Deserialize)]
    struct RecordEntry {
        name: String,
        zone: String,
        #[serde(default, deserialize_with = "bool_or_protocol")]
        v4: RecordLookup,
        #[serde(default, deserialize_with = "bool_or_protocol")]
        v6: RecordLookup,
    }

    let entries = Vec::<RecordEntry>::deserialize(d)?;
    let mut records = Records::default();
    for rec in entries {
        match rec.v4 {
            RecordLookup::Global => {
                records.v4.push(ZoneRecord {
                    zone: rec.zone.clone(),
                    name: rec.name.clone(),
                    lookup: None,
                });
            }
            RecordLookup::Custom(cfg) => {
                records.v4.push(ZoneRecord {
                    zone: rec.zone.clone(),
                    name: rec.name.clone(),
                    lookup: Some(cfg),
                });
            }
            RecordLookup::Disabled => {}
        }
        match rec.v6 {
            RecordLookup::Global => {
                records.v6.push(ZoneRecord {
                    zone: rec.zone.clone(),
                    name: rec.name.clone(),
                    lookup: None,
                });
            }
            RecordLookup::Custom(cfg) => {
                records.v6.push(ZoneRecord {
                    zone: rec.zone.clone(),
                    name: rec.name.clone(),
                    lookup: Some(cfg),
                });
            }
            RecordLookup::Disabled => {}
        }
    }
    Ok(records)
}
