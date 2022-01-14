use std::fmt;

use chrono::{DateTime, TimeZone, Utc};
use juniper::{
    execute, graphql_object, graphql_value, graphql_vars, DefaultScalarValue, EmptyMutation,
    EmptySubscription, GraphQLScalar, GraphQLType, InputValue, ParseScalarResult, ParseScalarValue,
    RootNode, ScalarToken, ScalarValue, Value,
};

fn schema<'q, C, Q>(query_root: Q) -> RootNode<'q, Q, EmptyMutation<C>, EmptySubscription<C>>
where
    Q: GraphQLType<DefaultScalarValue, Context = C, TypeInfo = ()> + 'q,
{
    RootNode::new(
        query_root,
        EmptyMutation::<C>::new(),
        EmptySubscription::<C>::new(),
    )
}

fn schema_with_scalar<'q, S, C, Q>(
    query_root: Q,
) -> RootNode<'q, Q, EmptyMutation<C>, EmptySubscription<C>, S>
where
    Q: GraphQLType<S, Context = C, TypeInfo = ()> + 'q,
    S: ScalarValue + 'q,
{
    RootNode::new_with_scalar_value(
        query_root,
        EmptyMutation::<C>::new(),
        EmptySubscription::<C>::new(),
    )
}

mod trivial_unnamed {
    use super::*;

    #[derive(GraphQLScalar)]
    struct Counter(i32);

    struct QueryRoot;

    #[graphql_object(scalar = DefaultScalarValue)]
    impl QueryRoot {
        fn counter(value: Counter) -> Counter {
            value
        }
    }

    #[tokio::test]
    async fn is_graphql_scalar() {
        const DOC: &str = r#"{
            __type(name: "Counter") {
                kind
            }
        }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"__type": {"kind": "SCALAR"}}), vec![])),
        );
    }

    #[tokio::test]
    async fn resolves_counter() {
        const DOC: &str = r#"{ counter(value: 0) }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"counter": 0}), vec![])),
        );
    }

    #[tokio::test]
    async fn has_no_description() {
        const DOC: &str = r#"{
            __type(name: "Counter") {
                description
            }
        }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"__type": {"description": null}}), vec![])),
        );
    }
}

mod trivial_named {
    use super::*;

    #[derive(GraphQLScalar)]
    struct Counter {
        value: i32,
    }

    struct QueryRoot;

    #[graphql_object(scalar = DefaultScalarValue)]
    impl QueryRoot {
        fn counter(value: Counter) -> Counter {
            value
        }
    }

    #[tokio::test]
    async fn is_graphql_scalar() {
        const DOC: &str = r#"{
            __type(name: "Counter") {
                kind
            }
        }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"__type": {"kind": "SCALAR"}}), vec![])),
        );
    }

    #[tokio::test]
    async fn resolves_counter() {
        const DOC: &str = r#"{ counter(value: 0) }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"counter": 0}), vec![])),
        );
    }

    #[tokio::test]
    async fn has_no_description() {
        const DOC: &str = r#"{
            __type(name: "Counter") {
                description
            }
        }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"__type": {"description": null}}), vec![])),
        );
    }
}

mod explicit_name {
    use super::*;

    #[derive(GraphQLScalar)]
    #[graphql(name = "Counter")]
    struct CustomCounter(i32);

    struct QueryRoot;

    #[graphql_object(scalar = DefaultScalarValue)]
    impl QueryRoot {
        fn counter(value: CustomCounter) -> CustomCounter {
            value
        }
    }

    #[tokio::test]
    async fn is_graphql_scalar() {
        const DOC: &str = r#"{
            __type(name: "Counter") {
                kind
            }
        }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"__type": {"kind": "SCALAR"}}), vec![])),
        );
    }

    #[tokio::test]
    async fn no_custom_counter() {
        const DOC: &str = r#"{
            __type(name: "CustomCounter") {
                kind
            }
        }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!(null), vec![])),
        );
    }

    #[tokio::test]
    async fn resolves_counter() {
        const DOC: &str = r#"{ counter(value: 0) }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"counter": 0}), vec![])),
        );
    }

    #[tokio::test]
    async fn has_no_description() {
        const DOC: &str = r#"{
            __type(name: "Counter") {
                description
            }
        }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"__type": {"description": null}}), vec![])),
        );
    }
}

mod custom_to_output {
    use super::*;

    #[derive(GraphQLScalar)]
    #[graphql(to_output_with = to_output)]
    struct Increment(i32);

    fn to_output<S: ScalarValue>(val: &Increment) -> Value<S> {
        let ret = val.0 + 1;
        ret.to_output()
    }

    struct QueryRoot;

    #[graphql_object(scalar = DefaultScalarValue)]
    impl QueryRoot {
        fn increment(value: Increment) -> Increment {
            value
        }
    }

    #[tokio::test]
    async fn is_graphql_scalar() {
        const DOC: &str = r#"{
            __type(name: "Increment") {
                kind
            }
        }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"__type": {"kind": "SCALAR"}}), vec![])),
        );
    }

    #[tokio::test]
    async fn resolves_counter() {
        const DOC: &str = r#"{ increment(value: 0) }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"increment": 1}), vec![])),
        );
    }

    #[tokio::test]
    async fn has_no_description() {
        const DOC: &str = r#"{
            __type(name: "Increment") {
                description
            }
        }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"__type": {"description": null}}), vec![])),
        );
    }
}

mod generic_with_all_resolvers {
    use super::*;

    #[derive(GraphQLScalar)]
    #[graphql(
        to_output_with = Self::to_output,
        from_input_with = Self::from_input,
        from_input_err = String,
    )]
    #[graphql(
        parse_token_with = Self::parse_token,
        specified_by_url = "https://tools.ietf.org/html/rfc3339"
    )]
    struct CustomDateTime<Tz>
    where
        Tz: From<Utc> + TimeZone,
        Tz::Offset: fmt::Display,
    {
        dt: DateTime<Tz>,
        _unused: (),
    }

    impl<S, Tz> GraphQLScalar<S> for CustomDateTime<Tz>
    where
        S: ScalarValue,
        Tz: From<Utc> + TimeZone,
        Tz::Offset: fmt::Display,
    {
        type Error = String;

        fn to_output(&self) -> Value<S> {
            Value::scalar(self.dt.to_rfc3339())
        }

        fn from_input(v: &InputValue<S>) -> Result<Self, Self::Error> {
            v.as_string_value()
                .ok_or_else(|| format!("Expected `String`, found: {}", v))
                .and_then(|s| {
                    DateTime::parse_from_rfc3339(s)
                        .map(|dt| Self {
                            dt: dt.with_timezone(&Tz::from(Utc)),
                            _unused: (),
                        })
                        .map_err(|e| format!("Failed to parse CustomDateTime: {}", e))
                })
        }

        fn parse_token(value: ScalarToken<'_>) -> ParseScalarResult<'_, S> {
            <String as ParseScalarValue<S>>::from_str(value)
        }
    }

    struct QueryRoot;

    #[graphql_object(scalar = DefaultScalarValue)]
    impl QueryRoot {
        fn date_time(value: CustomDateTime<Utc>) -> CustomDateTime<Utc> {
            value
        }
    }

    #[tokio::test]
    async fn resolves_custom_date_time() {
        const DOC: &str = r#"{ dateTime(value: "1996-12-19T16:39:57-08:00") }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((
                graphql_value!({"dateTime": "1996-12-20T00:39:57+00:00"}),
                vec![],
            )),
        );
    }

    #[tokio::test]
    async fn has_specified_by_url() {
        const DOC: &str = r#"{
            __type(name: "CustomDateTime") {
                specifiedByUrl
            }
        }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((
                graphql_value!({"__type": {"specifiedByUrl": "https://tools.ietf.org/html/rfc3339"}}),
                vec![],
            )),
        );
    }
}

mod description_from_doc_comment {
    use super::*;

    /// Doc comment.
    #[derive(GraphQLScalar)]
    struct Counter(i32);

    struct QueryRoot;

    #[graphql_object(scalar = DefaultScalarValue)]
    impl QueryRoot {
        fn counter(value: Counter) -> Counter {
            value
        }
    }

    #[tokio::test]
    async fn resolves_counter() {
        const DOC: &str = r#"{ counter(value: 0) }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"counter": 0}), vec![])),
        );
    }

    #[tokio::test]
    async fn has_description() {
        const DOC: &str = r#"{
            __type(name: "Counter") {
                description
            }
        }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((
                graphql_value!({"__type": {"description": "Doc comment."}}),
                vec![],
            )),
        );
    }
}

mod description_from_attribute {
    use super::*;

    /// Doc comment.
    #[derive(GraphQLScalar)]
    #[graphql(desc = "Doc comment from attribute.")]
    #[graphql(specified_by_url = "https://tools.ietf.org/html/rfc4122")]
    struct Counter(i32);

    struct QueryRoot;

    #[graphql_object(scalar = DefaultScalarValue)]
    impl QueryRoot {
        fn counter(value: Counter) -> Counter {
            value
        }
    }

    #[tokio::test]
    async fn resolves_counter() {
        const DOC: &str = r#"{ counter(value: 0) }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"counter": 0}), vec![])),
        );
    }

    #[tokio::test]
    async fn has_description_and_url() {
        const DOC: &str = r#"{
            __type(name: "Counter") {
                description
                specifiedByUrl
            }
        }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((
                graphql_value!({
                    "__type": {
                        "description": "Doc comment from attribute.",
                        "specifiedByUrl": "https://tools.ietf.org/html/rfc4122",
                    }
                }),
                vec![],
            )),
        );
    }
}

mod custom_scalar {
    use crate::custom_scalar::MyScalarValue;

    use super::*;

    #[derive(GraphQLScalar)]
    #[graphql(scalar = MyScalarValue)]
    struct Counter(i32);

    struct QueryRoot;

    #[graphql_object(scalar = MyScalarValue)]
    impl QueryRoot {
        fn counter(value: Counter) -> Counter {
            value
        }
    }

    #[tokio::test]
    async fn is_graphql_scalar() {
        const DOC: &str = r#"{
            __type(name: "Counter") {
                kind
            }
        }"#;

        let schema = schema_with_scalar::<MyScalarValue, _, _>(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"__type": {"kind": "SCALAR"}}), vec![])),
        );
    }

    #[tokio::test]
    async fn resolves_counter() {
        const DOC: &str = r#"{ counter(value: 0) }"#;

        let schema = schema_with_scalar::<MyScalarValue, _, _>(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"counter": 0}), vec![])),
        );
    }

    #[tokio::test]
    async fn has_no_description() {
        const DOC: &str = r#"{
            __type(name: "Counter") {
                description
            }
        }"#;

        let schema = schema_with_scalar::<MyScalarValue, _, _>(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"__type": {"description": null}}), vec![])),
        );
    }
}

mod generic_scalar {
    use super::*;

    #[derive(GraphQLScalar)]
    #[graphql(scalar = S: ScalarValue)]
    struct Counter(i32);

    struct QueryRoot;

    #[graphql_object]
    impl QueryRoot {
        fn counter(value: Counter) -> Counter {
            value
        }
    }

    #[tokio::test]
    async fn is_graphql_scalar() {
        const DOC: &str = r#"{
            __type(name: "Counter") {
                kind
            }
        }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"__type": {"kind": "SCALAR"}}), vec![])),
        );
    }

    #[tokio::test]
    async fn resolves_counter() {
        const DOC: &str = r#"{ counter(value: 0) }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"counter": 0}), vec![])),
        );
    }
}

mod bounded_generic_scalar {
    use super::*;

    #[derive(GraphQLScalar)]
    #[graphql(scalar = S: ScalarValue + Clone)]
    struct Counter(i32);

    struct QueryRoot;

    #[graphql_object(scalar = S: ScalarValue + Clone)]
    impl QueryRoot {
        fn counter(value: Counter) -> Counter {
            value
        }
    }

    #[tokio::test]
    async fn is_graphql_scalar() {
        const DOC: &str = r#"{
            __type(name: "Counter") {
                kind
            }
        }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"__type": {"kind": "SCALAR"}}), vec![])),
        );
    }

    #[tokio::test]
    async fn resolves_counter() {
        const DOC: &str = r#"{ counter(value: 0) }"#;

        let schema = schema(QueryRoot);

        assert_eq!(
            execute(DOC, None, &schema, &graphql_vars! {}, &()).await,
            Ok((graphql_value!({"counter": 0}), vec![])),
        );
    }
}
