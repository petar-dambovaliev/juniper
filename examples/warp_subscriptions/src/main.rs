//! This example demonstrates asynchronous subscriptions with warp and tokio 0.2

use std::{env, pin::Pin, sync::Arc, time::Duration};

use futures::{FutureExt as _, Stream};
use juniper::{
    graphql_object, graphql_subscription, graphql_value, EmptyMutation, FieldError, GraphQLEnum,
    RootNode,
};
use juniper_graphql_ws::ConnectionConfig;
use juniper_warp::{playground_filter, subscriptions::serve_graphql_ws};
use warp::{http::Response, Filter};

#[derive(Clone)]
struct Context;

impl juniper::Context for Context {}

#[derive(Clone, Copy, GraphQLEnum)]
enum UserKind {
    Admin,
    User,
    Guest,
}

struct User {
    id: i32,
    kind: UserKind,
    name: String,
}

// Field resolvers implementation
#[graphql_object(context = Context)]
impl User {
    fn id(&self) -> i32 {
        self.id
    }

    fn kind(&self) -> UserKind {
        self.kind
    }

    fn name(&self) -> &str {
        &self.name
    }

    async fn friends(&self) -> Vec<User> {
        if self.id == 1 {
            return vec![
                User {
                    id: 11,
                    kind: UserKind::User,
                    name: "user11".into(),
                },
                User {
                    id: 12,
                    kind: UserKind::Admin,
                    name: "user12".into(),
                },
                User {
                    id: 13,
                    kind: UserKind::Guest,
                    name: "user13".into(),
                },
            ];
        } else if self.id == 2 {
            return vec![User {
                id: 21,
                kind: UserKind::User,
                name: "user21".into(),
            }];
        } else if self.id == 3 {
            return vec![
                User {
                    id: 31,
                    kind: UserKind::User,
                    name: "user31".into(),
                },
                User {
                    id: 32,
                    kind: UserKind::Guest,
                    name: "user32".into(),
                },
            ];
        } else {
            return vec![];
        }
    }
}

struct Query;

#[graphql_object(context = Context)]
impl Query {
    async fn users(id: i32) -> Vec<User> {
        vec![User {
            id,
            kind: UserKind::Admin,
            name: "User Name".into(),
        }]
    }
}

type UsersStream = Pin<Box<dyn Stream<Item = Result<User, FieldError>> + Send>>;

struct Subscription;

#[graphql_subscription(context = Context)]
impl Subscription {
    async fn users() -> UsersStream {
        let mut counter = 0;
        let mut interval = tokio::time::interval(Duration::from_secs(5));
        let stream = async_stream::stream! {
            counter += 1;
            loop {
                interval.tick().await;
                if counter == 2 {
                    yield Err(FieldError::new(
                        "some field error from handler",
                        graphql_value!("some additional string"),
                    ))
                } else {
                    yield Ok(User {
                        id: counter,
                        kind: UserKind::Admin,
                        name: "stream user".into(),
                    })
                }
            }
        };

        Box::pin(stream)
    }
}

type Schema = RootNode<'static, Query, EmptyMutation<Context>, Subscription>;

fn schema() -> Schema {
    Schema::new(Query, EmptyMutation::new(), Subscription)
}

#[tokio::main]
async fn main() {
    env::set_var("RUST_LOG", "warp_subscriptions");
    env_logger::init();

    let log = warp::log("warp_subscriptions");

    let homepage = warp::path::end().map(|| {
        Response::builder()
            .header("content-type", "text/html")
            .body("<html><h1>juniper_subscriptions demo</h1><div>visit <a href=\"/playground\">graphql playground</a></html>")
    });

    let qm_schema = schema();
    let qm_state = warp::any().map(|| Context);
    let qm_graphql_filter = juniper_warp::make_graphql_filter(qm_schema, qm_state.boxed());

    let root_node = Arc::new(schema());

    log::info!("Listening on 127.0.0.1:8080");

    let routes = (warp::path("subscriptions")
        .and(warp::ws())
        .map(move |ws: warp::ws::Ws| {
            let root_node = root_node.clone();
            ws.on_upgrade(move |websocket| async move {
                serve_graphql_ws(websocket, root_node, ConnectionConfig::new(Context))
                    .map(|r| {
                        if let Err(e) = r {
                            println!("Websocket error: {e}");
                        }
                    })
                    .await
            })
        }))
    .map(|reply| {
        // TODO#584: remove this workaround
        warp::reply::with_header(reply, "Sec-WebSocket-Protocol", "graphql-ws")
    })
    .or(warp::post()
        .and(warp::path("graphql"))
        .and(qm_graphql_filter))
    .or(warp::get()
        .and(warp::path("playground"))
        .and(playground_filter("/graphql", Some("/subscriptions"))))
    .or(homepage)
    .with(log);

    warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;
}
