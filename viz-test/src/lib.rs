use reqwest::{Client, RequestBuilder};
use std::{net::SocketAddr, sync::Arc};
use tokio::net::TcpListener;
use viz::{server::conn::http1, Error, Responder, Result, Router, Tree};

pub struct TestServer {
    addr: SocketAddr,
    client: Client,
}

impl TestServer {
    /// Creates new test server
    /// 
    /// # Errors
    ///
    /// Will return `Err` if the server fails to start.
    pub async fn new(router: Router) -> Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let tree = Arc::new(Tree::from(router));
        let addr = listener.local_addr()?;
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(Error::normal)?;

        tokio::spawn(run(listener, tree));

        Ok(Self { addr, client })
    }

    fn path(&self, url: impl AsRef<str>) -> String {
        format!("http://{}{}", self.addr, url.as_ref())
    }

    #[must_use]
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    pub fn get(&self, url: impl AsRef<str>) -> RequestBuilder {
        self.client.get(self.path(url))
    }

    pub fn post(&self, url: impl AsRef<str>) -> RequestBuilder {
        self.client.post(self.path(url))
    }

    pub fn delete(&self, url: impl AsRef<str>) -> RequestBuilder {
        self.client.delete(self.path(url))
    }

    pub fn put(&self, url: impl AsRef<str>) -> RequestBuilder {
        self.client.put(self.path(url))
    }
}

async fn run(listener: TcpListener, tree: Arc<Tree>) -> Result<()> {
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
