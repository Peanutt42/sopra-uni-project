use serde::{Deserialize, Serialize};

// TODO: Human2
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Player {
	Human1,
	AI,
}
