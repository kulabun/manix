use crate::{
    contains_insensitive_ascii, starts_with_insensitive_ascii, Cache, DocEntry, DocSource, Errors,
    Lowercase,
};
use colored::*;
use serde::{Deserialize, Serialize, Deserializer};
use serde::ser::{Serializer, SerializeStruct};
use std::marker::PhantomData;
use std::{
    collections::HashMap,
    str::FromStr,
};
use std::path::PathBuf;
use void::Void;
use std::fmt;

use serde::de::{self, Visitor, MapAccess};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Description {
    #[serde(default)]
    pub text: String,
    #[serde(default, rename(deserialize = "_type"))]
    pub format: Option<String>,
}

impl FromStr for Description {
    // This implementation of `from_str` can never fail, so use the impossible
    // `Void` type as the error type.
    type Err = Void;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Description {
            text: s.to_string(),
            format: None,
        })
    }
}

impl Default for Description {
    fn default() -> Self {
        Description {
            text: String::new(),
            format: None,
        }
    }
}

fn string_or_struct<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: Default + Deserialize<'de> + FromStr<Err = Void>,
    D: Deserializer<'de>,
{
    // This is a Visitor that forwards string types to T's `FromStr` impl and
    // forwards map types to T's `Deserialize` impl. The `PhantomData` is to
    // keep the compiler from complaining about T being an unused generic type
    // parameter. We need T in order to know the Value type for the Visitor
    // impl.
    struct StringOrStruct<T>(PhantomData<fn() -> T>);

    impl<'de, T> Visitor<'de> for StringOrStruct<T>
    where
        T: Deserialize<'de> + FromStr<Err = Void>,
    {
        type Value = T;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("string or map")
        }

        fn visit_str<E>(self, value: &str) -> Result<T, E>
        where
            E: de::Error,
        {
            Ok(FromStr::from_str(value).unwrap())
        }

        fn visit_map<M>(self, map: M) -> Result<T, M::Error>
        where
            M: MapAccess<'de>,
        {
            // `MapAccessDeserializer` is a wrapper that turns a `MapAccess`
            // into a `Deserializer`, allowing it to be used as the input to T's
            // `Deserialize` implementation. T then deserializes itself using
            // the entries from the map visitor.
            Deserialize::deserialize(de::value::MapAccessDeserializer::new(map))
        }
    }

    Ok(deserializer.deserialize_any(StringOrStruct(PhantomData)).unwrap_or_default())
}


impl Serialize for Description {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("Description", 2)?;
        state.serialize_field("text", &self.text)?;
        state.serialize_field("_type", &self.format)?;
        state.end()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct JsonOptionDocumentation {
    // bincode really don't like when the field can be either string or struct, 
    // but I guess there may be a simlper solution rather than coping the struct
    #[serde(default, deserialize_with = "string_or_struct")]
    description: Description,

    #[serde(default, rename(serialize = "readOnly", deserialize = "readOnly"))]
    read_only: bool,

    #[serde(default,rename(serialize = "loc", deserialize = "loc"))]
    location: Vec<String>,

    #[serde(default, rename(serialize = "type", deserialize = "type"))]
    option_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OptionDocumentation {
    #[serde(default)]
    description: String,

    #[serde(default, rename(serialize = "readOnly", deserialize = "readOnly"))]
    read_only: bool,

    #[serde(default,rename(serialize = "loc", deserialize = "loc"))]
    location: Vec<String>,

    #[serde(default, rename(serialize = "type", deserialize = "type"))]
    option_type: String,
}

impl OptionDocumentation {
    pub fn name(&self) -> String {
        self.location.join(".")
    }
    pub fn pretty_printed(&self) -> String {
        format!(
            "# {}\n{}\ntype: {}\n\n",
            self.name().blue().bold(),
            self.description,
            self.option_type
        )
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptionsDatabaseType {
    NixOS,
    HomeManager,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OptionsDatabase {
    pub typ: OptionsDatabaseType,
    pub options: HashMap<String, OptionDocumentation>,
}

impl OptionsDatabase {
    pub fn new(typ: OptionsDatabaseType) -> Self {
        Self {
            typ,
            options: HashMap::new(),
        }
    }
}

pub fn try_from_file(path: &PathBuf) -> Result<HashMap<String, OptionDocumentation>, Errors> {
    let jsonOptions: HashMap<String, JsonOptionDocumentation> =
        serde_json::from_slice(&std::fs::read(path)?)?;
    let options = jsonOptions.into_iter()
                .map(|(k, v)| {
                    (
                        k,
                        OptionDocumentation {
                            description: v.description.text,
                            read_only: v.read_only,
                            location: v.location,
                            option_type: v.option_type,
                        },
                    )
                })
                .collect();
    Ok(options)
}

impl DocSource for OptionsDatabase {
    fn all_keys(&self) -> Vec<&str> {
        self.options.keys().map(|x| x.as_ref()).collect()
    }
    fn search(&self, query: &Lowercase) -> Vec<DocEntry> {
        self.options
            .iter()
            .filter(|(key, _)| starts_with_insensitive_ascii(key.as_bytes(), query))
            .map(|(_, d)| DocEntry::OptionDoc(self.typ, d.clone()))
            .collect()
    }
    fn search_liberal(&self, query: &Lowercase) -> Vec<DocEntry> {
        self.options
            .iter()
            .filter(|(key, _)| contains_insensitive_ascii(key.as_bytes(), query))
            .map(|(_, d)| DocEntry::OptionDoc(self.typ, d.clone()))
            .collect()
    }
    fn update(&mut self) -> Result<bool, Errors> {
        let opts = match self.typ {
            OptionsDatabaseType::NixOS => try_from_file(&get_nixos_json_doc_path()?)?,
            OptionsDatabaseType::HomeManager => try_from_file(&get_hm_json_doc_path()?)?,
        };

        let old = std::mem::replace(&mut self.options, opts);

        Ok(old.keys().eq(self.options.keys()))
    }
}

impl Cache for OptionsDatabase {}

pub fn get_hm_json_doc_path() -> Result<PathBuf, std::io::Error> {
    let hm_json_doc_path = std::env::var("HOME_MANAGER_JSON_OPTIONS_PATH")
        .expect("HOME_MANAGER_JSON_OPTIONS_PATH is not set");

    return Ok(PathBuf::from(hm_json_doc_path))
}

pub fn get_nixos_json_doc_path() -> Result<PathBuf, std::io::Error> {
    let nixos_json_doc_path = std::env::var("NIXOS_JSON_OPTIONS_PATH")
        .expect("NIXOS_JSON_OPTIONS_PATH is not set");

    return Ok(PathBuf::from(nixos_json_doc_path))
}
