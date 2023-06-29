use yew::platform::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;
use gloo_net::http::Request;

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/")]
    Home,
    #[at("/hello-server")]
    HelloServer,
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <Home /> },
        Route::HelloServer => html! { <HelloServer /> },
    }
}

#[function_component(Home)]
fn home() -> Html {
    html! {
        <div>
            <h1>{ "Home" }</h1>
            <Link<Route> to={Route::HelloServer} >{ "Hello Server!" }</Link<Route>>
        </div>
    }
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={switch} />
        </BrowserRouter>
    }
}


#[function_component(HelloServer)]
fn hello_server() -> Html {
    let data = use_state(|| None);
    {
        let data = data.clone();
        use_effect(move || {
            if data.is_none() {
                spawn_local(async move {
                    let resp = Request::get("/api/hello").send().await.unwrap();
                    let result = {
                        if !resp.ok() {
                            Err(format!("{} {}", resp.status(), resp.status_text()))
                        } else {
                            resp.text().await.map_err(|err| err.to_string())
                        }
                    };
                    data.set(Some(result));
                    log::info!("Got data");
                })
            }

            || {}
        });
    }

    match data.as_ref() {
        None => html! {
            <div>{ "Loading..." }</div>
        },
        Some(Ok(data)) => html! {
            <div>{ data }</div>
        },
        Some(Err(err)) => html! {
            <div>{ err }</div>
        },
    }
}

fn main() {
    wasm_logger::init(wasm_logger::Config::new(log::Level::Trace));
    console_error_panic_hook::set_once();
    yew::Renderer::<App>::new().render();
}
