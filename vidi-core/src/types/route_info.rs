/// Current route information.
#[derive(Debug)]
pub struct RouteInfo {
    /// Route ID.
    pub id: usize,
    /// Route pattern.
    pub pattern: String,
    /// Route params as name-value pairs.
    pub params: Vec<(String, String)>,
}

impl RouteInfo {
    /// Creates [`RouteInfo`] from borrowed route params.
    #[must_use]
    pub fn new(id: usize, pattern: String, params: Vec<(&str, &str)>) -> Self {
        Self {
            id,
            pattern,
            params: params
                .into_iter()
                .map(|(key, value)| (key.to_owned(), value.to_owned()))
                .collect(),
        }
    }
}
