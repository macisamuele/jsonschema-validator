setup_loader![crate::types::testing::TestingType, TestingType, TestingLoader, (), |content: String| Ok(
    if content.is_empty() { TestingType::Null } else { TestingType::from(content) }
),];
