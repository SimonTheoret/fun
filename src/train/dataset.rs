use serde::{Deserialize, Serialize};

use crate::{InputEvent, Timestamp};
use derive_more;
use flatarray::FlatArray;
use std::fs::File;

#[derive(derive_more::Deref, derive_more::DerefMut, Serialize, Deserialize)]
pub struct Dataset<T> {
    #[deref]
    #[deref_mut]
    array: FlatArray<(InputEvent, Timestamp)>,
    #[serde(skip)]
    source: T,
}

type FileDataset = Dataset<std::fs::File>;


impl Dataset<T> {
    pub fn new(source:T) -> Self {
        let arr = FlatArray::default();
        Self {
            array: arr, 
                   source
        }
    }
}
