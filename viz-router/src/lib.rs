//! Router for Viz Web Framework

#![doc(html_logo_url = "https://viz.rs/logo.svg")]
#![doc(html_favicon_url = "https://viz.rs/logo.svg")]
#![doc(test(
    no_crate_inject,
    attr(deny(warnings, rust_2018_idioms), allow(dead_code, unused_variables))
))]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![allow(clippy::needless_pass_by_value)]

#[macro_use]
pub(crate) mod macros;

mod resources;
pub use resources::Resources;

mod route;
pub use route::*;

mod router;
pub use router::Router;

mod tree;
pub use tree::Tree;

pub use path_tree::{Path, PathTree};
