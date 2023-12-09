//! A collection of handlers for Viz.

#![doc(html_logo_url = "https://viz.rs/logo.svg")]
#![doc(html_favicon_url = "https://viz.rs/logo.svg")]
#![doc(test(
    no_crate_inject,
    attr(
        deny(warnings, rust_2018_idioms),
        allow(dead_code, unused_assignments, unused_variables)
    )
))]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

#[cfg(feature = "serve")]
pub mod serve;

#[cfg(feature = "embed")]
pub mod embed;

#[cfg(feature = "prometheus")]
pub mod prometheus;
