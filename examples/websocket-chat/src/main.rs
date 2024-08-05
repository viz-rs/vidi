use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::sync::broadcast::{channel, Sender};
use viz::{
    get, serve,
    types::{Message, Params, State, WebSocket},
    HandlerExt, IntoHandler, IntoResponse, Request, RequestExt, Response, ResponseExt, Result,
    Router,
};

async fn index() -> Result<Response> {
    Ok(Response::html::<&'static str>(include_str!(
        "../index.html"
    )))
}

async fn ws(mut req: Request) -> Result<impl IntoResponse> {
    let (ws, Params(name), State(sender)): (WebSocket, Params<String>, State<Sender<String>>) =
        req.extract().await?;

    let tx = sender.clone();
    let mut rx = sender.subscribe();

    Ok(ws.on_upgrade(move |socket| async move {
        // Split the socket into a sender and receive of messages.
        let (mut ws_tx, mut ws_rx) = socket.split();

        tokio::task::spawn(async move {
            while let Ok(msg) = rx.recv().await {
                if ws_tx.send(Message::Text(msg)).await.is_err() {
                    break;
                }
            }
        });

        while let Some(Ok(msg)) = ws_rx.next().await {
            if let Message::Text(text) = msg {
                // Maybe should check user name, dont send to current user.
                if tx.send(format!("{name}: {text}")).is_err() {
                    break;
                }
            }
        }

        println!("websocket was closed");
    }))
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(addr).await?;
    println!("listening on http://{addr}");

    let channel = channel::<String>(32);

    let app = Router::new()
        .route("/", get(index.into_handler()))
        .route("/ws/:name", get(ws.with(State::new(channel.0))));

    if let Err(e) = serve(listener, app).await {
        println!("{e}");
    }

    Ok(())
}
