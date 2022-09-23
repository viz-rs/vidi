/// Current routing information
#[derive(Clone, Debug)]
pub struct Route {
    /// Route ID
    pub id: usize,
    /// Route Path
    pub path: String,
}

impl Route {
    /// Creates a new route information.
    pub fn new(id: usize, path: String) -> Self {
        Self { id, path }
    }
}
