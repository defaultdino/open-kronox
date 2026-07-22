use std::collections::HashMap;

use serde::Deserialize;

use crate::error::Error;

#[derive(Debug, Clone, Deserialize)]
pub struct School {
    pub id: String,
    pub name: String,
    pub domain: String,
    pub urls: Vec<String>,
    #[serde(rename = "logoUrl", default)]
    pub logo_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SchoolsConfig {
    pub schools: HashMap<String, School>,
}

impl SchoolsConfig {
    /// Resolve config from the environment or a file, in that order.
    ///
    /// # Errors
    /// Returns [`Error::Config`] if no source is found or the JSON is invalid or empty.
    pub fn load() -> Result<Self, Error> {
        if let Ok(json) = std::env::var("KRONOX_SCHOOLS_JSON") {
            return Self::from_json(&json);
        }
        let path = std::env::var("KRONOX_SCHOOLS_FILE")
            .unwrap_or_else(|_| ".well-known/schools.json".to_owned());
        let contents = std::fs::read_to_string(&path)
            .map_err(|e| Error::Config(format!("reading {path}: {e}")))?;
        Self::from_json(&contents)
    }

    fn from_json(json: &str) -> Result<Self, Error> {
        let config: SchoolsConfig =
            serde_json::from_str(json).map_err(|e| Error::Config(e.to_string()))?;
        if config.schools.is_empty() {
            return Err(Error::Config("no schools configured".to_owned()));
        }
        Ok(config)
    }

    #[must_use]
    pub fn get(&self, code: &str) -> Option<&School> {
        self.schools.get(code)
    }

    #[must_use]
    pub fn max_url_index(&self, code: &str) -> Option<usize> {
        self.get(code)
            .map(|school| school.urls.len().saturating_sub(1))
    }

    #[must_use]
    pub fn allowed(&self) -> Vec<String> {
        self.schools.keys().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_and_indexes() {
        let json = r#"{"schools":{"hkr":{"id":"hkr","name":"Kristianstad","domain":"hkr.se","urls":["https://schema.hkr.se/","https://kronox.hkr.se/"]}}}"#;
        let config = SchoolsConfig::from_json(json).unwrap();
        assert_eq!(config.max_url_index("hkr"), Some(1));
        assert_eq!(config.max_url_index("nope"), None);
        assert_eq!(config.get("hkr").unwrap().urls.len(), 2);
    }

    #[test]
    fn rejects_empty() {
        assert!(SchoolsConfig::from_json(r#"{"schools":{}}"#).is_err());
    }
}
