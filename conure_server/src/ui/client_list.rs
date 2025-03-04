use crate::client_manager::ClientMap;
use dioxus::prelude::*;

#[component]
pub fn ClientList() -> Element {
    let clients = use_context::<Signal<ClientMap>>();

    rsx! {
        div {
            h2 { "Client List" }
            div {
                for item in clients.read().values() {
                    div {
                        p { "{item.identifier()}" }
                    }
                }
            }
        }
    }
}
