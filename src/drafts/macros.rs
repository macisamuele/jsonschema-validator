macro_rules! initialize_keywords {
    (
        primitive_type: $primitive_type: ident,
        scoped_schema: $scoped_schema: expr,
        path: $path: expr,
        raw_schema: $raw_schema: expr,
        keywords: [$($keyword_type:ident,)*],
    ) => {{
        use std::sync::Arc;
        use crate::keywords::KeywordAttribute;
        use crate::keywords::KeywordTrait;
        use crate::schema::ValidationError;

        let attribute_names: Vec<&str> = iterate![[$($keyword_type::<$primitive_type>::ATTRIBUTE,)*]].filter_map(
            |&attribute_name| {
                if $raw_schema.has_attribute(attribute_name) {
                    Some(attribute_name)
                } else {
                    None
                }
            }
        ).collect();

        let (ok_keywords, err_optional_validation_errors): (
            Vec<Result<Arc<KeywordTrait<T>>, _>>,
            Vec<Result<_, Option<ValidationError<T>>>>,
        ) = attribute_names.iter().filter_map(  // FIXME: re-enable parallel execution
            |&attribute_name| {
                match attribute_name {
                    $(
                        $keyword_type::<$primitive_type>::ATTRIBUTE => {
                            if $keyword_type::<$primitive_type>::is_keyword($raw_schema) {
                                Some($keyword_type::<$primitive_type>::new($scoped_schema, $path.clone(), Box::new($raw_schema.clone())).and_then(
                                    |keyword| Ok({
                                        let _thing: Arc<KeywordTrait<T>> = Arc::new(keyword);
                                        _thing
                                    })
                                ))
                            } else {
                                None
                            }
                        },
                    )*
                    _ => None,
                }
            }
        ).partition(Result::is_ok);

        let validation_errors_lists: Vec<ValidationError<T>> = into_iterator![err_optional_validation_errors].filter_map(Result::unwrap_err).collect();
        let keywords: Vec<Arc<KeywordTrait<T>>> = if validation_errors_lists.is_empty() {
            into_iterator![ok_keywords].map(Result::unwrap).collect()
        } else {
            vec![]
        };
        (keywords, validation_errors_lists)
    }}
}
