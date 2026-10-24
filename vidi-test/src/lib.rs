use reqwest::Client;
use std::{future::IntoFuture, net::SocketAddr};
use tokio::net::TcpListener;
use vidi::{Error, Result, Router, serve};

pub use http;
pub use nano_id;
pub use sessions;

pub use reqwest::{RequestBuilder, multipart};

pub struct TestServer {
    addr: SocketAddr,
    client: Client,
}

macro_rules! http_method {
    ($name:ident) => {
        pub fn $name(&self, url: impl AsRef<str>) -> RequestBuilder {
            self.client.$name(self.path(url))
        }
    };
}

impl TestServer {
    /// Starts a test server with the given router.
    ///
    /// # Errors
    ///
    /// Returns an error if the server fails to start.
    pub async fn new(router: Router) -> Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(Error::boxed)?;

        tokio::spawn(serve(listener, router).into_future());

        Ok(Self { addr, client })
    }

    fn path(&self, url: impl AsRef<str>) -> String {
        format!("http://{}{}", self.addr, url.as_ref())
    }

    #[must_use]
    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    http_method!(get);
    http_method!(post);
    http_method!(delete);
    http_method!(put);
}
