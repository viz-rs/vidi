use std::fmt::{Debug, Formatter, Result};

use path_tree::{Path, PathTree};

use viz_core::{BoxHandler, Method};

use crate::{Route, Router};

/// Store all final routes.
#[derive(Default)]
pub struct Tree(Vec<(Method, PathTree<BoxHandler>)>);

impl Tree {
    /// Find a handler by the HTTP method and the URI's path.
    pub fn find<'a, 'b>(
        &'a self,
        method: &'b Method,
        path: &'b str,
    ) -> Option<(&'a BoxHandler, Path<'a, 'b>)> {
        self.0
            .iter()
            .find_map(|(m, t)| if m == method { t.find(path) } else { None })
    }

    /// Consumes the Tree, returning the wrapped value.
    pub fn into_inner(self) -> Vec<(Method, PathTree<BoxHandler>)> {
        self.0
    }
}

impl AsRef<Vec<(Method, PathTree<BoxHandler>)>> for Tree {
    fn as_ref(&self) -> &Vec<(Method, PathTree<BoxHandler>)> {
        &self.0
    }
}

impl AsMut<Vec<(Method, PathTree<BoxHandler>)>> for Tree {
    fn as_mut(&mut self) -> &mut Vec<(Method, PathTree<BoxHandler>)> {
        &mut self.0
    }
}

impl From<Router> for Tree {
    fn from(router: Router) -> Self {
        let mut tree = Tree::default();
        if let Some(routes) = router.routes {
            for (mut path, Route { methods }) in routes {
                if !path.starts_with('/') {
                    path.insert(0, '/');
                }
                for (method, handler) in methods {
                    match tree.as_mut().iter_mut().find_map(|(m, t)| {
                        if *m == method {
                            Some(t)
                        } else {
                            None
                        }
                    }) {
                        Some(t) => {
                            let _ = t.insert(&path, handler);
                        }
                        None => {
                            let mut t = PathTree::new();
                            let _ = t.insert(&path, handler);
                            tree.as_mut().push((method, t));
                        }
                    }
                }
            }
        }
        tree
    }
}

impl Debug for Tree {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        self.as_ref()
            .iter()
            .fold(f.debug_struct("Tree"), |mut d, (m, t)| {
                d.field("method", m).field("paths", &t.node);
                d
            })
            .finish()
    }
}
