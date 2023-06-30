use crate::image_future::ImageFuture;
use gloo::utils::document;
use gloo_net::http::Request;
use log::info;
use yew::platform::spawn_local;
use yew::prelude::*;
use yew_router::prelude::*;

mod image_future;
mod text_input;

use crate::text_input::TextInput;

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
            <h1>{ "Rusty Graph" }</h1>
            // <Link<Route> to={Route::HelloServer} >{ "Hello Server!" }</Link<Route>>
            <Search />
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

#[derive(Clone, PartialEq, Properties)]
struct PlotProps {
    name: String,
}

#[function_component(Plot)]
fn plot(props: &PlotProps) -> Html {
    let is_loaded = use_state(|| false);

    let contents = use_state(|| {
        let div: web_sys::Element = document().create_element("div").unwrap();
        div.set_inner_html("Loading...");
        let node: web_sys::Node = div.into();
        Html::VRef(node)
    });
    {
        let contents = contents.clone();
        let is_loaded = is_loaded.clone();
        use_effect_with_deps(
            move |name| {
                // show loader
                let div: web_sys::Element = document().create_element("div").unwrap();
                div.set_inner_html("Loading...");
                let node: web_sys::Node = div.into();
                contents.set(Html::VRef(node));

                let name = name.clone();
                info!("fetch image: {}", name);
                spawn_local(async move {
                    let name = urlencoding::encode(&name);
                    let url = format!("/api/image?name={}", name);
                    let image = ImageFuture::new(&url).await.unwrap();
                    info!("done");
                    let node: web_sys::Node = image.into();
                    contents.set(Html::VRef(node));
                    is_loaded.set(true);
                });
            },
            props.name.clone(),
        );
    }

    (*contents).clone()
    // <img src={format!("/api/image?name={name}")} />
}

#[function_component(Search)]
fn search() -> Html {
    let search = use_state(|| "".to_string());
    let name = use_state(|| -> Option<String> { None });

    let on_change = {
        let search = search.clone();
        Callback::from(move |s| search.set(s))
    };

    let onsubmit = {
        let search = search.clone();
        let name = name.clone();
        Callback::from(move |s: SubmitEvent| {
            s.prevent_default();
            info!("Submit: {}", *search);
            name.set(Some((*search).clone()));
        })
    };

    let name = (*name).clone();

    html! {
        <div>
        { "Search: " }
        <form method="post" {onsubmit}>
        <TextInput value={(*search).clone()} on_change={on_change} />
        </form>
        if let Some(name) = name {
            <Plot {name} />
        }
        </div>
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
                    info!("Got data");
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
