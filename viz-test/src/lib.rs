use reqwest::Client;
use std::{future::IntoFuture, net::SocketAddr};
use tokio::net::TcpListener;
use viz::{serve, Error, Result, Router};

pub use http;
pub use nano_id;
pub use sessions;

pub use reqwest::{multipart, RequestBuilder};

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
