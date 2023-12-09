# Examples for Viz

Here you can find a lot of small crabs 🦀.

## Table of contents

* [Hello world](hello-world)
* [Unix socket domain](unix-socket)
* [Static file serving and directory listing](static-files/serve)
* [Static files serving and embedding](static-files/embed)
* [Extract body from Form](forms/form)
* [Extract body from Multipart](forms/multipart)
* [Extract body data with a limits](limits)
* [Websockt Chat](websocket-chat)
* [Server-Sent Events](sse)
* [Session](session)
* [CSRF](csrf)
* [CORS](cors)
* [Compression response body](compression)
* [HTTPS/TLS - rustls](rustls)
* [Defined a static router](static-routes)
* [Todos](routing/todos)
* [OpenAPI](routing/openapi) powered by [utoipa](https://docs.rs/utoipa/latest/utoipa/)
* [Integration Opentelemetry(OTEL)](https://github.com/open-telemetry/opentelemetry-rust)
  * [Tracing](otel/tracing)
  * [Metrics & Prometheus](otel/metrics)
* [Template](templates)
  * [askama](templates/askama)
  * [markup](templates/markup)
  * [tera](templates/tera)
* [Tracing aka logging](tracing)

## Usage

### Run it in `viz` directory

```console
$ cargo run --bin hello-world -- --nocapture
```

### Fetch data

```console
$ curl http://127.0.0.1:3000
```
