use dioxus::prelude::*;
use dioxus_router::prelude::*;

use crate::ui::Route;

#[component]
pub fn Wrapper() -> Element {
    rsx! {
        main {
            class: "container-fluid",
            nav {
                ul {
                    li {
                        strong { "Conure" }
                    }
                }
                ul {
                    li {
                        Link { to: Route::ClientList, "Client List" }
                    }
                }
            }
            Outlet::<Route> {}
        }
    }
}
