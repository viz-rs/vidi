//! web api mod

use crate::entities::todo::{ActiveModel, Entity as todoEntity, Model as Todo};

use sea_orm::{
    ActiveModelTrait, ActiveValue::NotSet, DatabaseConnection, EntityTrait, Set, TryIntoModel,
};
use vidi::{
    IntoResponse, Request, RequestExt, Response, ResponseExt, Result,
    types::{Json, Params, State},
};

/// list todos
/// # Errors
/// - `vidi::Error`
pub async fn list(mut req: Request) -> Result<Response> {
    let State(db) = req.extract::<State<DatabaseConnection>>().await?;
    let todos = todoEntity::find()
        .all(&db)
        .await
        .map_err(|err| err.to_string().into_error())?;
    Ok(Response::json(todos)?)
}

/// create todos
/// # Errors
/// - `vidi::Error`
pub async fn create(mut req: Request) -> Result<Response> {
    let (State(db), Json(todo)) = req
        .extract::<(State<DatabaseConnection>, Json<Todo>)>()
        .await?;

    let mut todo_am: ActiveModel = todo.into();
    todo_am.id = NotSet;
    let result = todo_am
        .insert(&db)
        .await
        .map_err(|err| err.to_string().into_error())?;
    let todo_new: Todo = result
        .try_into_model()
        .map_err(|err| err.to_string().into_error())?;
    Ok(Response::json(todo_new)?)
}

/// update todos
/// PUT /todos/:id
/// # Errors
/// - `vidi::Error`
pub async fn update(mut req: Request) -> Result<Response> {
    let (State(db), Params(id), Json(todo)) = req
        .extract::<(State<DatabaseConnection>, Params<i32>, Json<Todo>)>()
        .await?;
    let mut todo_am: ActiveModel = todo.clone().into();
    todo_am.id = Set(id);
    todo_am.completed = Set(todo.completed);
    let model = todo_am
        .update(&db)
        .await
        .map_err(|err| err.to_string().into_error())?;

    Ok(Response::json(model)?)
}

/// delete todos
/// DELETE /todos/:id
/// # Errors
/// - `vidi::Error`
pub async fn delete(mut req: Request) -> Result<Response> {
    let (State(db), Params(id)) = req
        .extract::<(State<DatabaseConnection>, Params<i32>)>()
        .await?;
    let delete_result = todoEntity::delete_by_id(id)
        .exec(&db)
        .await
        .map_err(|err| err.to_string().into_error())?;
    let rows_affected = delete_result.rows_affected;
    Ok(Response::json(rows_affected)?)
}
