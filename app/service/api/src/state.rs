use std::sync::Arc;

use crate::{flatgeobuf::FgbFileSource, regions::Regions, routing::StadiaMapsRouting};

#[derive(Clone)]
pub struct AppState {
    pub flatgeobuf: Arc<FgbFileSource>,
    pub regions: Arc<Regions>,
    pub routing: Arc<StadiaMapsRouting>,
}
