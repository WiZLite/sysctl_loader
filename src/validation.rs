use std::collections::{HashMap, HashSet};

use crate::types::{Schema, SchemaType, SysctlValue, ValidationError};

pub fn validate_by_schema(
    value: &HashMap<String, SysctlValue>,
    schema: &Schema,
) -> Result<(), Vec<ValidationError>> {
    let value_keys: HashSet<&String> = value.keys().into_iter().collect();
    let schema_keys: HashSet<&String> = schema.entries.iter().map(|entry| &entry.name).collect();
    let missing_keys = schema_keys.difference(&value_keys);
    let unknown_keys = value_keys.difference(&schema_keys);
    let common_keys = schema_keys.union(&value_keys);
    let mut wrong_types = Vec::new();
    for common_key in common_keys {
        if let Some(schema_entry) = schema
            .entries
            .iter()
            .find(|entry| &entry.name == *common_key)
        {
            let expected_type = schema_entry.schema_type;
            if let Some(sysctl_value) = value.get(*common_key) {
                let actual_type = SchemaType::from_str(&sysctl_value.value);
                match expected_type {
                    SchemaType::String => {
                        // boolやnumber形式であったとしても、stringとして許可する
                        // 4096文字を超える行長がないかどうかだけチェックする
                        if sysctl_value
                            .value
                            .lines()
                            .any(|line| line.chars().count() >= 4096)
                        {
                            wrong_types.push(ValidationError::TooLongLine(common_key.to_string()))
                        }
                    }
                    SchemaType::Boolean | SchemaType::Number => {
                        if schema_entry.schema_type != actual_type {
                            wrong_types.push(ValidationError::WrongType {
                                key_name: common_key.to_string(),
                                expect: schema_entry.schema_type,
                                actual: actual_type,
                            });
                        }
                    }
                }
            }
        }
    }

    let mut errors = Vec::new();
    errors.extend(
        missing_keys
            .into_iter()
            .map(|x| ValidationError::MissingKey(x.to_string()))
            .collect::<Vec<_>>(),
    );
    errors.extend(
        unknown_keys
            .into_iter()
            .map(|x| ValidationError::UnknownKey(x.to_string()))
            .collect::<Vec<_>>(),
    );
    errors.extend(wrong_types);

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use crate::types::SchemaEntry;

    impl PartialOrd for ValidationError {
        fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
            let key_name = match self {
                ValidationError::MissingKey(key_name) => key_name,
                ValidationError::UnknownKey(key_name) => key_name,
                ValidationError::WrongType { key_name, .. } => key_name,
                ValidationError::TooLongLine(key_name) => key_name,
            };
            let other_key_name = match other {
                ValidationError::MissingKey(key_name) => key_name,
                ValidationError::UnknownKey(key_name) => key_name,
                ValidationError::WrongType { key_name, .. } => key_name,
                ValidationError::TooLongLine(key_name) => key_name,
            };

            Some(key_name.cmp(other_key_name))
        }
    }
    impl Ord for ValidationError {
        fn cmp(&self, other: &Self) -> std::cmp::Ordering {
            self.partial_cmp(other).unwrap()
        }
    }

    use super::*;
    #[test]
    fn validate_by_schema_success() {
        let value: HashMap<String, SysctlValue> = [
            (
                "key1".to_string(),
                SysctlValue {
                    value: "value1".to_string(),
                    ignore_error: false,
                },
            ),
            (
                "key2".to_string(),
                SysctlValue {
                    value: "false".to_string(),
                    ignore_error: false,
                },
            ),
            (
                "key3".to_string(),
                SysctlValue {
                    value: "3.14".to_string(),
                    ignore_error: false,
                },
            ),
        ]
        .into_iter()
        .collect();

        let schema = Schema {
            entries: vec![
                SchemaEntry {
                    name: "key1".to_string(),
                    schema_type: SchemaType::String,
                },
                SchemaEntry {
                    name: "key2".to_string(),
                    schema_type: SchemaType::Boolean,
                },
                SchemaEntry {
                    name: "key3".to_string(),
                    schema_type: SchemaType::Number,
                },
            ],
        };

        let result = validate_by_schema(&value, &schema);
        assert!(result.is_ok());
    }

    #[test]
    fn validate_by_schema_error() {
        assert_eq!(
            validate_by_schema(
                &[
                    (
                        "key1".to_string(),
                        SysctlValue {
                            // valid as string
                            value: "true".to_string(),
                            ignore_error: false,
                        },
                    ),
                    (
                        "key2".to_string(),
                        SysctlValue {
                            value: "true?".to_string(),
                            ignore_error: false,
                        },
                    ),
                    (
                        "key3".to_string(),
                        SysctlValue {
                            value: "3..14".to_string(),
                            ignore_error: false,
                        },
                    ),
                ]
                .into_iter()
                .collect(),
                &Schema {
                    entries: vec![
                        SchemaEntry {
                            name: "key1".to_string(),
                            schema_type: SchemaType::String,
                        },
                        SchemaEntry {
                            name: "key2".to_string(),
                            schema_type: SchemaType::Boolean,
                        },
                        SchemaEntry {
                            name: "key3".to_string(),
                            schema_type: SchemaType::Number,
                        },
                    ],
                }
            )
            .map_err(|errors| {
                let mut es = errors.clone();
                es.sort();
                es
            }),
            Err(vec![
                ValidationError::WrongType {
                    key_name: "key2".to_string(),
                    expect: SchemaType::Boolean,
                    actual: SchemaType::String,
                },
                ValidationError::WrongType {
                    key_name: "key3".to_string(),
                    expect: SchemaType::Number,
                    actual: SchemaType::String,
                },
            ],)
        );
        // Checking missing keys
        assert_eq!(
            validate_by_schema(
                &[(
                    "key1".to_string(),
                    SysctlValue {
                        // valid as string
                        value: "true".to_string(),
                        ignore_error: false,
                    },
                ),]
                .into_iter()
                .collect(),
                &Schema {
                    entries: vec![
                        SchemaEntry {
                            name: "key1".to_string(),
                            schema_type: SchemaType::String,
                        },
                        SchemaEntry {
                            name: "key2".to_string(),
                            schema_type: SchemaType::Boolean,
                        },
                    ],
                }
            ),
            Err(vec![ValidationError::MissingKey("key2".to_string())])
        );
        // Checking unknown keys
        assert_eq!(
            validate_by_schema(
                &[
                    (
                        "key1".to_string(),
                        SysctlValue {
                            // valid as string
                            value: "true".to_string(),
                            ignore_error: false,
                        },
                    ),
                    (
                        "key2".to_string(),
                        SysctlValue {
                            value: "true?".to_string(),
                            ignore_error: false,
                        },
                    ),
                ]
                .into_iter()
                .collect(),
                &Schema {
                    entries: vec![SchemaEntry {
                        name: "key1".to_string(),
                        schema_type: SchemaType::String,
                    }]
                }
            ),
            Err(vec![ValidationError::UnknownKey("key2".to_string())])
        )
    }
}
