use std::sync::Arc;

use crate::{regions::Regions, routing::StadiaMapsRouting};

#[derive(Clone)]
pub struct AppState {
    pub regions: Arc<Regions>,
    pub routing: Arc<StadiaMapsRouting>,
}
