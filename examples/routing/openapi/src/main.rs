#![deny(warnings)]
#![allow(clippy::unused_async)]
#![allow(clippy::needless_pass_by_value)]

use std::{
    net::SocketAddr,
    sync::{Arc, Mutex, PoisonError},
};
use tokio::net::TcpListener;

use http_body_util::Full;
use serde::{Deserialize, Serialize};
use utoipa::{
    openapi::security::{ApiKey, ApiKeyValue, SecurityScheme},
    Modify, OpenApi, ToSchema,
};
use utoipa_swagger_ui::Config;
use viz::{
    header::{self, HeaderMap},
    headers::HeaderValue,
    middleware,
    server::conn::http1,
    types::{Json, Params, Query, State, StateError},
    Error, HandlerExt, IntoResponse, Request, RequestExt, Responder, Response, ResponseExt, Result,
    Router, StatusCode, Tree,
};

/// In-memory todo store
type DB = Arc<Mutex<Vec<Todo>>>;

const LIMIT: usize = 10;

fn into_error<T>(e: PoisonError<T>) -> Error {
    e.to_string().into_error()
}

#[derive(Serialize, Deserialize, ToSchema, Clone)]
struct Todo {
    pub id: u64,
    #[schema(example = "Buy groceries")]
    pub text: String,
    pub completed: bool,
}

#[derive(Serialize, Deserialize, ToSchema)]
enum TodoError {
    /// Todo already exists conflict.
    #[schema(example = "Todo already exists")]
    Conflict(String),
    /// Todo not found by id.
    #[schema(example = "id = 1")]
    NotFound(String),
    /// Todo operation unauthorized
    #[schema(example = "missing api key")]
    Unauthorized(String),
}

// The query parameters for list todos.
#[derive(Debug, Deserialize)]
struct Pagination {
    pub offset: Option<usize>,
    pub limit: Option<usize>,
}

// GET /todos?offset=0&limit=10
#[utoipa::path(
    get,
    path = "/todos",
    responses(
        (status = 200, description = "List all todos successfully", body = [Todo])
    )
)]
async fn list(mut req: Request) -> Result<Response> {
    let (State(db), Query(Pagination { offset, limit })) =
        req.extract::<(State<DB>, Query<Pagination>)>().await?;

    let todos = db
        .lock()
        .map_err(into_error)?
        .iter()
        .skip(offset.unwrap_or(0))
        .take(limit.unwrap_or(LIMIT))
        .cloned()
        .collect::<Vec<Todo>>();

    Ok(Response::json(todos)?)
}

// POST /todos
#[utoipa::path(
    post,
    path = "/todos",
    request_body = Todo,
    responses(
        (status = 201, description = "Todo item created successfully", body = Todo),
        (status = 409, description = "Todo already exists", body = TodoError)
    )
)]
async fn create(mut req: Request) -> Result<Response> {
    let (State(db), Json(todo)) = req.extract::<(State<DB>, Json<Todo>)>().await?;

    let mut todos = db.lock().map_err(into_error)?;

    if todos.iter().any(|t| t.id == todo.id) {
        return Ok((
            StatusCode::CONFLICT,
            Json(TodoError::Conflict(format!(
                "todo already exists: {}",
                todo.id
            ))),
        )
            .into_response());
    }

    todos.push(todo.clone());

    Ok((StatusCode::CREATED, Json(todo)).into_response())
}

// GET /todos/:id
#[utoipa::path(
    post,
    path = "/todos/{id}",
    responses(
        (status = 200, description = "Todo item found successfully", body = Todo),
        (status = 404, description = "Todo not found")
    ),
    params(
        ("id" = u64, Path, description = "Todo database id")
    ),
    security(
        (), // <-- make optional authentication
        ("api_key" = [])
    )
)]
async fn show(mut req: Request) -> Result<Response> {
    let (State(db), Params(id)) = req.extract::<(State<DB>, Params<u64>)>().await?;

    let todo = db
        .lock()
        .map_err(into_error)?
        .iter()
        .find(|t| t.id == id)
        .cloned()
        .ok_or_else(|| StatusCode::NOT_FOUND.into_error())?;

    Ok(Response::json(todo)?)
}

// PUT /todos/:id
#[utoipa::path(
    put,
    path = "/todos/{id}",
    responses(
        (status = 200, description = "Todo marked done successfully"),
        (status = 404, description = "Todo not found")
    ),
    params(
        ("id" = u64, Path, description = "Todo database id")
    ),
    security(
        (), // <-- make optional authentication
        ("api_key" = [])
    )
)]
async fn update(mut req: Request) -> Result<StatusCode> {
    check_api_key(false, req.headers())?;

    let (State(db), Params(id), Json(todo)) = req
        .extract::<(State<DB>, Params<u64>, Json<Todo>)>()
        .await?;

    let mut todos = db.lock().map_err(into_error)?;

    for t in todos.iter_mut() {
        if t.id == id {
            *t = todo;
            return Ok(StatusCode::OK);
        }
    }

    Ok(StatusCode::NOT_FOUND)
}

// DELETE /todos/:id
#[utoipa::path(
    delete,
    path = "/todos/{id}",
    responses(
        (status = 200, description = "Todo marked done successfully"),
        (status = 401, description = "Unauthorized to delete Todo", body = TodoError, example = json!(TodoError::Unauthorized(String::from("missing api key")))),
        (status = 404, description = "Todo not found", body = TodoError, example = json!(TodoError::NotFound(String::from("id = 1"))))
    ),
    params(
        ("id" = u64, Path, description = "Todo database id")
    ),
    security(
        ("api_key" = [])
    )
)]
async fn delete(mut req: Request) -> Result<StatusCode> {
    check_api_key(true, req.headers())?;

    let (State(db), Params(id)) = req.extract::<(State<DB>, Params<u64>)>().await?;

    let mut todos = db.lock().map_err(into_error)?;

    let len = todos.len();
    todos.retain(|t| t.id != id);

    // not found todo by id
    if todos.len() == len {
        return Ok(StatusCode::NOT_FOUND);
    }

    Ok(StatusCode::NO_CONTENT)
}

// normally you should create a middleware for this but this is sufficient for sake of example.
fn check_api_key(require_api_key: bool, headers: &HeaderMap<HeaderValue>) -> Result<()> {
    match headers.get("todo_apikey") {
        Some(header) if header != "utoipa-rocks" => Err((
            StatusCode::UNAUTHORIZED,
            Json(TodoError::Unauthorized(String::from("incorrect api key"))),
        )
            .into_error()),
        None if require_api_key => Err((
            StatusCode::UNAUTHORIZED,
            Json(TodoError::Unauthorized(String::from("missing api key"))),
        )
            .into_error()),
        _ => Ok(()),
    }
}

#[derive(OpenApi)]
#[openapi(
    paths(
        list,
        create,
        update,
        delete,
    ),
    components(
        schemas(Todo, TodoError)
    ),
    modifiers(&SecurityAddon),
    tags(
        (name = "todo", description = "Todo items management API")
    )
)]
struct ApiDoc;

struct SecurityAddon;

impl Modify for SecurityAddon {
    fn modify(&self, openapi: &mut utoipa::openapi::OpenApi) {
        if let Some(components) = openapi.components.as_mut() {
            components.add_security_scheme(
                "api_key",
                SecurityScheme::ApiKey(ApiKey::Header(ApiKeyValue::new("todo_apikey"))),
            );
        }
    }
}

async fn openapi_json(req: Request) -> Result<Response> {
    let apidoc = req
        .state::<Arc<utoipa::openapi::OpenApi>>()
        .ok_or_else(StateError::new::<Arc<utoipa::openapi::OpenApi>>)?;
    Ok(Response::json(&*apidoc)?)
}

async fn swagger_ui(req: Request) -> Result<Response> {
    let config = req
        .state::<Arc<Config>>()
        .ok_or_else(StateError::new::<Arc<Config>>)?;
    let tail = req
        .route_info()
        .params
        .first()
        .map_or_else(|| "", |(_, p)| p.as_str());

    match utoipa_swagger_ui::serve(tail, config)
        .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()).into_error())?
    {
        Some(file) => Ok({
            let content_type = HeaderValue::from_str(&file.content_type).map_err(Error::normal)?;

            let mut resp = Response::new(Full::from(file.bytes).into());
            resp.headers_mut().insert(header::CONTENT_TYPE, content_type);
            resp
        }),
        None => Err(StatusCode::NOT_FOUND.into_error()),
    }
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;
    println!("listening on {addr}");

    let apidoc = Arc::new(ApiDoc::openapi());
    let config = Arc::new(Config::from("/api-doc/openapi.json"));

    let db = DB::default();

    let app = Router::new()
        .get("/todos", list)
        .post("/todos", create)
        .get("/todos/:id", show)
        .put("/todos/:id", update)
        .delete("/todos/:id", delete)
        .with(State::new(db))
        // Set limits for the payload data of request
        .with(middleware::limits::Config::new())
        .get(
            "/api-doc/openapi.json",
            openapi_json.with(State::new(apidoc)),
        )
        .get("/swagger-ui/*", swagger_ui.with(State::new(config)));
    let tree = Arc::new(Tree::from(app));

    loop {
        let (stream, addr) = listener.accept().await?;
        let tree = tree.clone();
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(stream, Responder::new(tree, Some(addr)))
                .await
            {
                eprintln!("Error while serving HTTP connection: {err}");
            }
        });
    }
}
