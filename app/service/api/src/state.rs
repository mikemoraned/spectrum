use std::sync::Arc;

use crate::{flatgeobuf::FgbSource, regions::Regions, routing::StadiaMapsRouting};

#[derive(Clone)]
pub struct AppState {
    pub flatgeobuf: Arc<FgbSource>,
    pub regions: Arc<Regions>,
    pub routing: Arc<StadiaMapsRouting>,
}
