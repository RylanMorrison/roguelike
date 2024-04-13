mod logstore;
mod builder;
mod events;

use serde::{Serialize, Deserialize};
pub use logstore::*;
pub use builder::*;
pub use events::*;

use rltk::RGB;

#[derive(Serialize, Deserialize, Clone)]
pub struct LogFragment {
    pub colour: RGB,
    pub text: String
}
