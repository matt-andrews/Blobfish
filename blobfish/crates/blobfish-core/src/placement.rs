// blobfish-core/src/lib.rs
pub trait PlacementEngine: Send + Sync {
    fn pick_nodes(&self, replication_factor: usize) -> Vec<String>;
}

pub struct LocalPlacement {
    node_id: String,
}

impl LocalPlacement {
    pub fn new(node_id: impl Into<String>) -> Self {
        Self { node_id: node_id.into() }
    }
}

impl PlacementEngine for LocalPlacement {
    fn pick_nodes(&self, _replication_factor: usize) -> Vec<String> {
        vec![self.node_id.clone()]
    }
}