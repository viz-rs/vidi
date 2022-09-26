<p align="center">
  <img src="https://raw.githubusercontent.com/viz-rs/viz.rs/main/static/logo.svg" height="200" />
</p>

<h1 align="center">
  <a href="https://docs.rs/viz">Viz</a>
</h1>

<div align="center">
  <p><strong>Core Components for Viz</strong></p>
</div>

<div align="center">
  <!-- Safety -->
  <a href="/">
    <img src="https://img.shields.io/badge/-safety!-success?style=flat-square"
      alt="Safety!" /></a>
  <!-- Docs.rs docs -->
  <a href="https://docs.rs/viz-core">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square"
      alt="Docs.rs docs" /></a>
  <!-- Crates version -->
  <a href="https://crates.io/crates/viz-core">
    <img src="https://img.shields.io/crates/v/viz-core.svg?style=flat-square"
    alt="Crates.io version" /></a>
  <!-- Downloads -->
  <a href="https://crates.io/crates/viz-core">
    <img src="https://img.shields.io/crates/d/viz-core.svg?style=flat-square"
      alt="Download" /></a>
</div>


## Built-in Extractors

Extractor   | Description
----------- | ------------
[Cookies]   | Extracts the `cookies` from the request.
[Form]      | Extracts `from-data` from the body of a request.
[Header]    | Extracts a `header` from the headers of a request.
[Json]      | Extracts `JSON` data from the body of a request, or responds a JSON data to response.
[Limits]    | Extracts the `limits` settings.
[Multipart] | Extracts the data from the `multipart` body of a request.
[Params]    | Extracts `params` from the path of a URL.
[Query]     | Extracts the data from the `query string` of a URL.
[Session]   | A `session` for the current request.
[State]     | Extracts `state` from the extensions of a request.
[Websocket] | A `WebSocket` connection.

[Query]: https://docs.rs/viz-core/latest/viz_core/types/struct.Query.html
[Params]: https://docs.rs/viz-core/latest/viz_core/types/struct.Params.html
[Header]: https://docs.rs/viz-core/latest/viz_core/types/struct.Header.html
[Cookies]: https://docs.rs/viz-core/latest/viz_core/types/struct.Cookies.html
[Form]: https://docs.rs/viz-core/latest/viz_core/types/struct.Form.html
[Json]: https://docs.rs/viz-core/latest/viz_core/types/struct.Json.html
[Multipart]: https://docs.rs/viz-core/latest/viz_core/types/type.Multipart.html
[Session]: https://docs.rs/viz-core/latest/viz_core/types/struct.Session.html
[State]: https://docs.rs/viz-core/latest/viz_core/types/struct.State.html
[Websocket]: https://docs.rs/viz-core/latest/viz_core/types/struct.WebSocket.html
[Limits]: https://docs.rs/viz-core/latest/viz_core/types/struct.Limits.html

## Built-in Middleware

Middleware                       | Description
-------------------------------- | ------------
[cookie][m:cookie]               | Cookie
[cors][m:cors]                   | CORS
[csrf][m:csrf]                   | CSRF
[limits][m:limits]               | Limits
[session][m:session]             | Session
[otel::tracing][m:otel::tracing] | OpenTelemetry Tracing
[otel::metrics][m:otel::metrics] | OpenTelemetry Metrics

[m:cookie]: https://docs.rs/viz-core/latest/viz_core/middleware/cookie
[m:cors]: https://docs.rs/viz-core/latest/viz_core/middleware/cors
[m:csrf]: https://docs.rs/viz-core/latest/viz_core/middleware/csrf
[m:limits]: https://docs.rs/viz-core/latest/viz_core/middleware/limits
[m:session]: https://docs.rs/viz-core/latest/viz_core/middleware/session
[m:otel::tracing]: https://docs.rs/viz-core/latest/viz_core/middleware/otel/tracing
[m:otel::metrics]: https://docs.rs/viz-core/latest/viz_core/middleware/otel/metrics


## License

Licensed under either of [Apache License, Version 2.0](LICENSE-APACHE) or
[MIT license](LICENSE-MIT) at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted 
for inclusion in Viz by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
