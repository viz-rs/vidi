use crate::types::Params;

/// Current route information.
#[derive(Debug)]
pub struct RouteInfo {
    /// Route ID
    pub id: usize,
    /// Route Pattern
    pub pattern: String,
    /// Route Params
    pub params: Params,
}
