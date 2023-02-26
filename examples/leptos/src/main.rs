#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() -> viz::Result<()> {
    use leptos::*;
    use leptos_start::app::*;
    use leptos_viz::{generate_route_list, handle_server_fns, LeptosRoutes};
    use std::sync::Arc;
    use viz::{handlers::serve, types::State, Router, Server, ServiceMaker};

    let conf = get_configuration(None).await.unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let site_root = &leptos_options.site_root;
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(|cx| view! { cx, <App/> }).await;

    let app = Router::new()
        .get("/api/:tail*", handle_server_fns)
        .leptos_routes(leptos_options.clone(), routes, |cx| view! { cx, <App/> })
        .get("/*", serve::Dir::new(site_root))
        .with(State(Arc::new(leptos_options)));

    if let Err(err) = Server::bind(&addr).serve(ServiceMaker::from(app)).await {
        println!("{err}");
    }

    Ok(())
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for pure client-side testing
    // see lib.rs for hydration function instead
}
