use handlebars::{DirectorySourceOptions, Handlebars};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{
    env,
    net::SocketAddr,
    path::PathBuf,
    sync::{Arc, LazyLock, Mutex, PoisonError},
};
use tokio::net::TcpListener;
use viz::{
    header::HeaderValue, middleware::limits, serve, types::State, Error, IntoResponse, Request,
    RequestExt, Response, ResponseExt, Result, Router, StatusCode,
};

/// In-memory todo store
type DB = Arc<Mutex<Vec<Todo>>>;

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Todo {
    pub text: String,
    pub completed: bool,
}

static TPLS: LazyLock<Handlebars> = LazyLock::new(|| {
    let dir = env::var("CARGO_MANIFEST_DIR").map(PathBuf::from).unwrap();
    let mut h = Handlebars::new();
    h.register_templates_directory(dir.join("templates"), DirectorySourceOptions::default())
        .unwrap();
    h
});

#[allow(clippy::needless_pass_by_value)]
fn into_error<T>(e: PoisonError<T>) -> Error {
    e.to_string().into_error()
}

async fn index(req: Request) -> Result<Response> {
    let todos = req
        .state::<DB>()
        .unwrap()
        .lock()
        .map_err(into_error)?
        .clone();
    let body = TPLS
        .render(
            "index",
            &json!({
                "todos": todos
            }),
        )
        .map_err(Error::boxed)?;
    Ok(Response::html(body))
}

async fn list(req: Request) -> Result<Response> {
    let todos = req
        .state::<DB>()
        .unwrap()
        .lock()
        .map_err(into_error)?
        .clone();
    let body = TPLS
        .render(
            "todos",
            &json!({
                "todos": todos
            }),
        )
        .map_err(Error::boxed)?;
    Ok(Response::html(body))
}

async fn create(mut req: Request) -> Result<Response> {
    let todo = req.form::<Todo>().await?;
    let db = req.state::<DB>().unwrap();

    let mut todos = db.lock().map_err(into_error)?;
    todos.push(todo);

    let mut resp = StatusCode::CREATED.into_response();
    resp.headers_mut()
        .insert("HX-Trigger", HeaderValue::from_static("newTodo"));
    Ok(resp)
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;
    println!("listening on http://{addr}");

    let app = Router::new()
        .get("/", index)
        .get("/todos", list)
        .post("/todos", create)
        .any("/*", |_| async { Ok(Response::text("Welcome!")) })
        .with(State::new(DB::default()))
        .with(limits::Config::default());

    if let Err(e) = serve(listener, app).await {
        println!("{e}");
    }

    Ok(())
}
