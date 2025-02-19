use schemars::{gen::SchemaSettings, schema::RootSchema};
pub use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize, Serializer};

/// The format to return a response in
#[derive(Debug, Clone)]
pub enum FormatType {
    Json,

    /// Requires Ollama 0.5.0 or greater.
    StructuredJson(JsonStructure),
}

impl Serialize for FormatType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            FormatType::Json => serializer.serialize_str("json"),
            FormatType::StructuredJson(s) => s.serialize(serializer),
        }
    }
}

/// Represents a serialized JSON schema. You can create this by converting
/// a JsonSchema:
/// ```rust
/// let json_schema = schema_for!(Output);
/// let serialized: SerializedJsonSchema = json_schema.into();
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct JsonStructure {
    schema: RootSchema,
}

impl Serialize for JsonStructure {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use serde::ser::SerializeMap;

        // First serialize to Value to handle the conversion
        let value = serde_json::to_value(&self.schema).map_err(serde::ser::Error::custom)?;

        // Create a modified version with $defs
        let mut modified_value = value
            .as_object()
            .ok_or_else(|| serde::ser::Error::custom("Schema root must be an object"))?
            .clone();

        // Replace definitions with $defs if it exists
        if let Some(definitions) = modified_value.remove("definitions") {
            modified_value.insert("$defs".to_string(), definitions);
        }

        // Manually serialize the modified map
        let mut map = serializer.serialize_map(Some(modified_value.len()))?;
        for (k, v) in modified_value {
            map.serialize_entry(&k, &v)?;
        }
        map.end()
    }
}

impl JsonStructure {
    pub fn new<T: JsonSchema>() -> Self {
        // Need to do this because Ollama doesn't support $refs (references in the schema)
        // So we have to explicitly turn them off
        let mut settings = SchemaSettings::draft07();
        settings.inline_subschemas = true;
        let generator = settings.into_generator();
        let schema = generator.into_root_schema_for::<T>();

        Self { schema }
    }

    /// Creates a JsonStructure from a RootSchema. Warning: Ollama doesn't support $ref in schemas.
    /// Use JsonStructure::new() instead for automatic subschema inlining.
    pub fn from_schema(schema: RootSchema) -> Self {
        Self { schema }
    }
}

/// Used to control how long a model stays loaded in memory, by default models are unloaded after 5 minutes of inactivity
#[derive(Debug, Clone)]
pub enum KeepAlive {
    Indefinitely,
    UnloadOnCompletion,
    Until { time: u64, unit: TimeUnit },
}

impl Serialize for KeepAlive {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            KeepAlive::Indefinitely => serializer.serialize_i8(-1),
            KeepAlive::UnloadOnCompletion => serializer.serialize_i8(0),
            KeepAlive::Until { time, unit } => {
                let mut s = String::new();
                s.push_str(&time.to_string());
                s.push_str(unit.to_symbol());
                serializer.serialize_str(&s)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum TimeUnit {
    Seconds,
    Minutes,
    Hours,
}

impl TimeUnit {
    pub fn to_symbol(&self) -> &'static str {
        match self {
            TimeUnit::Seconds => "s",
            TimeUnit::Minutes => "m",
            TimeUnit::Hours => "hr",
        }
    }
}
