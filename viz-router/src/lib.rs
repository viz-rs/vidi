//! Router for Viz Web Framework

#![doc(html_logo_url = "https://viz.rs/logo.svg")]
#![doc(html_favicon_url = "https://viz.rs/logo.svg")]
#![forbid(unsafe_code)]
#![warn(
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    unreachable_pub
)]
#![doc(test(
    no_crate_inject,
    attr(
        deny(warnings, rust_2018_idioms),
        allow(dead_code, unused_assignments, unused_variables)
    )
))]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]

#[macro_use]
pub(crate) mod macros;

mod resources;
mod route;
mod router;
mod tree;

pub use path_tree::{Path, PathTree};
pub use resources::Resources;
pub use route::*;
pub use router::Router;
pub use tree::Tree;
