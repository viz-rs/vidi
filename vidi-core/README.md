<p align="center">
  <img src="https://raw.githubusercontent.com/viz-rs/viz-rs.github.io/gh-pages/logo.svg" height="200" />
</p>

<h1 align="center">
  <a href="https://docs.rs/vidi">Vidi</a>
</h1>

<div align="center">
  <p><strong>Core Components for Vidi</strong></p>
</div>

<div align="center">
  <!-- Safety -->
  <a href="/">
    <img src="https://img.shields.io/badge/-safety!-success?style=flat-square"
      alt="Safety!" /></a>
  <!-- Docs.rs docs -->
  <a href="https://docs.rs/vidi-core">
    <img src="https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square"
      alt="Docs.rs docs" /></a>
  <!-- Crates version -->
  <a href="https://crates.io/crates/vidi-core">
    <img src="https://img.shields.io/crates/v/vidi-core.svg?style=flat-square"
    alt="Crates.io version" /></a>
  <!-- Downloads -->
  <a href="https://crates.io/crates/vidi-core">
    <img src="https://img.shields.io/crates/d/vidi-core.svg?style=flat-square"
      alt="Download" /></a>
</div>

## Built-in Extractors

| Extractor   | Description                                                                           |
| ----------- | ------------------------------------------------------------------------------------- |
| [Cookies]   | Extracts the `cookies` from the request.                                              |
| [Form]      | Extracts `from-data` from the body of a request.                                      |
| [Header]    | Extracts a `header` from the headers of a request.                                    |
| [Json]      | Extracts `JSON` data from the body of a request, or responds a JSON data to response. |
| [Limits]    | Extracts the `limits` settings.                                                       |
| [Multipart] | Extracts the data from the `multipart` body of a request.                             |
| [Params]    | Extracts `params` from the path of a URL.                                             |
| [Query]     | Extracts the data from the `query string` of a URL.                                   |
| [Session]   | A `session` for the current request.                                                  |
| [State]     | Extracts `state` from the extensions of a request.                                    |
| [Websocket] | A `WebSocket` connection.                                                             |

[query]: https://docs.rs/vidi-core/latest/vidi_core/types/struct.Query.html
[params]: https://docs.rs/vidi-core/latest/vidi_core/types/struct.Params.html
[header]: https://docs.rs/vidi-core/latest/vidi_core/types/struct.Header.html
[cookies]: https://docs.rs/vidi-core/latest/vidi_core/types/struct.Cookies.html
[form]: https://docs.rs/vidi-core/latest/vidi_core/types/struct.Form.html
[json]: https://docs.rs/vidi-core/latest/vidi_core/types/struct.Json.html
[multipart]: https://docs.rs/vidi-core/latest/vidi_core/types/type.Multipart.html
[session]: https://docs.rs/vidi-core/latest/vidi_core/types/struct.Session.html
[state]: https://docs.rs/vidi-core/latest/vidi_core/types/struct.State.html
[websocket]: https://docs.rs/vidi-core/latest/vidi_core/types/struct.WebSocket.html
[limits]: https://docs.rs/vidi-core/latest/vidi_core/types/struct.Limits.html

## Built-in Middleware

| Middleware                       | Description           |
| -------------------------------- | --------------------- |
| [cookie][m:cookie]               | Cookie                |
| [cors][m:cors]                   | CORS                  |
| [csrf][m:csrf]                   | CSRF                  |
| [limits][m:limits]               | Limits                |
| [session][m:session]             | Session               |
| [compression][m:compression]     | Compression           |
| [otel::tracing][m:otel::tracing] | OpenTelemetry Tracing |
| [otel::metrics][m:otel::metrics] | OpenTelemetry Metrics |

[m:cookie]: https://docs.rs/vidi-core/latest/vidi_core/middleware/cookie
[m:cors]: https://docs.rs/vidi-core/latest/vidi_core/middleware/cors
[m:csrf]: https://docs.rs/vidi-core/latest/vidi_core/middleware/csrf
[m:limits]: https://docs.rs/vidi-core/latest/vidi_core/middleware/limits
[m:session]: https://docs.rs/vidi-core/latest/vidi_core/middleware/session
[m:compression]: https://docs.rs/vidi-core/latest/vidi_core/middleware/compression
[m:otel::tracing]: https://docs.rs/vidi-core/latest/vidi_core/middleware/otel/tracing
[m:otel::metrics]: https://docs.rs/vidi-core/latest/vidi_core/middleware/otel/metrics

## License

This project is licensed under the [MIT license](LICENSE).

## Author

- [@fundon@fosstodon.org](https://fosstodon.org/@fundon)

- [@\_fundon](https://twitter.com/_fundon)
