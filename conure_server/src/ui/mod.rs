use axum::{Router, extract::ws::WebSocketUpgrade, response::Html, routing::get};
use dioxus::prelude::*;
use dioxus_router::prelude::{Routable, Router};
use layout::Wrapper;
use log::info;

use crate::client_manager::ClientManager;
use client_list::ClientList;

mod client_list;
mod layout;

#[derive(Clone, Debug, PartialEq, Routable)]
#[rustfmt::skip]
enum Route {
    #[layout(Wrapper)]
    #[route("/")]
    ClientList,
}

#[derive(Clone)]
struct AppContext {
    manager: ClientManager,
}

fn app(app_context: AppContext) -> Element {
    let manager = use_context_provider(|| app_context.clone()).manager;
    let mut clients = use_context_provider(|| Signal::new(manager.clients()));

    use_future(move || {
        let manager = app_context.manager.clone();
        async move {
            while let Ok(updated_clients) = manager.get_listener().recv().await {
                clients.set(updated_clients);
            }
        }
    });

    rsx! { Router::<Route> {} }
}

pub async fn ui_main(client_manager: ClientManager) {
    let addr: std::net::SocketAddr = ([127, 0, 0, 1], 3030).into();
    let view = dioxus_liveview::LiveViewPool::new();
    let app_context = AppContext {
        manager: client_manager,
    };

    let app = Router::new()
        .route(
            "/",
            get(move || async move {
                Html(format!(
                    include_str!("index.html"),
                    glue = dioxus_liveview::interpreter_glue(&format!("ws://{addr}/ws"))
                ))
            }),
        )
        .route(
            "/ws",
            get(move |ws: WebSocketUpgrade| async move {
                ws.on_upgrade(move |socket| async move {
                    _ = view
                        .launch_with_props(dioxus_liveview::axum_socket(socket), app, app_context)
                        .await;
                })
            }),
        );

    info!("Listening on http://{addr}");

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
