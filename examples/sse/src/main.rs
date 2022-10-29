#![deny(warnings)]

use futures_util::StreamExt;
use std::{net::SocketAddr, sync::Arc};
use systemstat::{Platform, System};
use tokio::net::TcpListener;
use tokio::time::{interval, Duration};
use tokio_stream::wrappers::IntervalStream;
use viz::{
    get,
    header::ACCEPT,
    server::conn::http1,
    types::{Event, Sse, State},
    Error, HandlerExt, IntoResponse, Request, RequestExt, Responder, Response, ResponseExt, Result,
    Router, StatusCode, Tree,
};

type ArcSystem = Arc<System>;

async fn index(_: Request) -> Result<Response> {
    Ok(Response::html::<&'static str>(include_str!(
        "../index.html"
    )))
}

async fn stats(req: Request) -> Result<impl IntoResponse> {
    // check request `Accept` header
    if !matches!(req.header::<_, String>(ACCEPT), Some(ts) if ts == mime::TEXT_EVENT_STREAM.as_ref())
    {
        Err(StatusCode::BAD_REQUEST.into_error())?
    }

    let sys = req
        .state::<ArcSystem>()
        .ok_or_else(|| StatusCode::INTERNAL_SERVER_ERROR.into_error())?;

    Ok(Sse::new(
        IntervalStream::new(interval(Duration::from_secs(10))).map(move |_| {
            match sys
                .load_average()
                .map_err(Error::normal)
                .and_then(|loadavg| serde_json::to_string(&loadavg).map_err(Error::normal))
            {
                Ok(loadavg) => Event::default().data(loadavg),
                Err(err) => {
                    println!("{err}");
                    Event::default().retry(30)
                }
            }
        }),
    )
    .interval(Duration::from_secs(15)))
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;
    println!("listening on {addr}");

    let sys = Arc::new(System::new());

    let app = Router::new()
        .route("/", get(index))
        .route("/stats", get(stats.with(State::new(sys))));
    let tree = Arc::new(Tree::from(app));

    loop {
        let (stream, addr) = listener.accept().await?;
        let tree = tree.clone();
        tokio::task::spawn(async move {
            if let Err(err) = http1::Builder::new()
                .serve_connection(stream, Responder::new(tree, Some(addr)))
                .await
            {
                eprintln!("Error while serving HTTP connection: {}", err);
            }
        });
    }
}
