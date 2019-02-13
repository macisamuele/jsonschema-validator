#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Copy, EnumIter, Eq, Debug, Display, PartialEq)]
pub enum EnumPrimitiveType {
    // We assume that all the drafts will have the same primitive types
    Array,
    Boolean,
    Integer,
    Null,
    Number,
    Object,
    String,
}

impl EnumPrimitiveType {
    #[inline]
    pub fn from_type(type_string: &str) -> Option<Self>
    where
        Self: Sized,
    {
        match type_string {
            "array" => Some(EnumPrimitiveType::Array),
            "boolean" => Some(EnumPrimitiveType::Boolean),
            "integer" => Some(EnumPrimitiveType::Integer),
            "null" => Some(EnumPrimitiveType::Null),
            "number" => Some(EnumPrimitiveType::Number),
            "object" => Some(EnumPrimitiveType::Object),
            "string" => Some(EnumPrimitiveType::String),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::EnumPrimitiveType;
    use test_case_derive::test_case;

    #[test_case("array", Some(EnumPrimitiveType::Array))]
    #[test_case("integer", Some(EnumPrimitiveType::Integer))]
    #[test_case("number", Some(EnumPrimitiveType::Number))]
    #[test_case("null", Some(EnumPrimitiveType::Null))]
    #[test_case("object", Some(EnumPrimitiveType::Object))]
    #[test_case("string", Some(EnumPrimitiveType::String))]
    #[test_case("an invalid type", None)]
    fn test_enum_primitive_type(type_str: &str, expected_option_enum_primitive_type: Option<EnumPrimitiveType>) {
        assert_eq!(EnumPrimitiveType::from_type(type_str), expected_option_enum_primitive_type);
    }
}
