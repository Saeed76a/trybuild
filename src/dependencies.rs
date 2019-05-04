use crate::error::Error;
use serde::de::value::MapAccessDeserializer;
use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::BTreeMap as Map;
use std::fmt;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use toml::Value;

pub fn get(manifest_dir: &Path) -> Map<String, Dependency> {
    try_get(manifest_dir).unwrap_or_default()
}

fn try_get(manifest_dir: &Path) -> Result<Map<String, Dependency>, Error> {
    let cargo_toml_path = manifest_dir.join("Cargo.toml");
    let manifest_str = fs::read_to_string(cargo_toml_path)?;
    let manifest: Manifest = toml::from_str(&manifest_str)?;

    let mut dependencies = manifest.dev_dependencies;
    dependencies.remove("trybuild");

    for dep in dependencies.values_mut() {
        dep.path = dep.path.as_ref().map(|path| manifest_dir.join(path));
    }

    Ok(dependencies)
}

#[derive(Deserialize)]
struct Manifest {
    #[serde(default, rename = "dev-dependencies")]
    dev_dependencies: Map<String, Dependency>,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(remote = "Self")]
pub struct Dependency {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,
    #[serde(flatten)]
    pub rest: Map<String, Value>,
}

impl Serialize for Dependency {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        Dependency::serialize(self, serializer)
    }
}

impl<'de> Deserialize<'de> for Dependency {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct DependencyVisitor;

        impl<'de> Visitor<'de> for DependencyVisitor {
            type Value = Dependency;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str(
                    "a version string like \"0.9.8\" or a \
                     dependency like { version = \"0.9.8\" }",
                )
            }

            fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Dependency {
                    version: Some(s.to_owned()),
                    path: None,
                    rest: Map::new(),
                })
            }

            fn visit_map<M>(self, map: M) -> Result<Self::Value, M::Error>
            where
                M: de::MapAccess<'de>,
            {
                Dependency::deserialize(MapAccessDeserializer::new(map))
            }
        }

        deserializer.deserialize_any(DependencyVisitor)
    }
}
