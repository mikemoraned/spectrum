use std::sync::Arc;

use crate::regions::Regions;

#[derive(Clone)]
pub struct AppState {
    pub regions: Arc<Regions>,
}
