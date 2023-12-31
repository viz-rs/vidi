#![deny(warnings)]

use std::{
    net::SocketAddr,
    sync::{Arc, Mutex, PoisonError},
};
use tokio::net::TcpListener;

use serde::{Deserialize, Serialize};
use viz::{
    middleware, serve,
    types::{Json, Params, Query, State},
    Error, IntoResponse, Request, RequestExt, Response, ResponseExt, Result, Router, StatusCode,
};

/// In-memory todo store
type DB = Arc<Mutex<Vec<Todo>>>;

const LIMIT: usize = 10;

#[allow(clippy::needless_pass_by_value)]
fn into_error<T>(e: PoisonError<T>) -> Error {
    e.to_string().into_error()
}

#[derive(Debug, Clone, Deserialize, Serialize)]
struct Todo {
    pub id: u64,
    pub text: String,
    pub completed: bool,
}

// The query parameters for list todos.
#[derive(Debug, Deserialize)]
struct Pagination {
    pub offset: Option<usize>,
    pub limit: Option<usize>,
}

// GET /todos?offset=0&limit=10
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
async fn create(mut req: Request) -> Result<StatusCode> {
    let (State(db), Json(todo)) = req.extract::<(State<DB>, Json<Todo>)>().await?;

    let mut todos = db.lock().map_err(into_error)?;

    if todos.iter().any(|t| t.id == todo.id) {
        return Ok(StatusCode::BAD_REQUEST);
    }

    todos.push(todo);

    Ok(StatusCode::CREATED)
}

// GET /todos/:id
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
async fn update(mut req: Request) -> Result<StatusCode> {
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
async fn delete(mut req: Request) -> Result<StatusCode> {
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

#[tokio::main]
async fn main() -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;
    println!("listening on http://{addr}");

    let db = DB::default();

    let app = Router::new()
        .get("/todos", list)
        .post("/todos", create)
        .get("/todos/:id", show)
        .put("/todos/:id", update)
        .delete("/todos/:id", delete)
        .with(State::new(db))
        // Set limits for the payload data of request
        .with(middleware::limits::Config::new());

    if let Err(e) = serve(listener, app).await {
        println!("{e}");
    }

    Ok(())
}
