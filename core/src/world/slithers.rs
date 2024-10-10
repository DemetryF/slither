use std::collections::HashMap;
use std::ops::{Index, IndexMut};

use serde::{Deserialize, Serialize};

use macros::id;

use crate::Slither;

#[id]
pub struct SlitherID;

#[derive(Default, Serialize, Deserialize)]
pub struct Slithers {
    data: HashMap<SlitherID, Slither>,
}

impl Slithers {
    pub fn add(&mut self, id: SlitherID, slither: Slither) {
        self.data.insert(id, slither).unwrap();
    }

    pub fn get(&self, id: SlitherID) -> &Slither {
        &self.data[&id]
    }

    pub fn remove(&mut self, id: SlitherID) -> Slither {
        self.data.remove(&id).unwrap()
    }

    pub fn iter(&self) -> impl Iterator<Item = (SlitherID, &Slither)> {
        self.data.iter().map(|(&id, slither)| (id, slither))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (SlitherID, &mut Slither)> {
        self.data.iter_mut().map(|(&id, slither)| (id, slither))
    }
}

impl Index<SlitherID> for Slithers {
    type Output = Slither;

    fn index(&self, id: SlitherID) -> &Self::Output {
        &self.data[&id]
    }
}

impl IndexMut<SlitherID> for Slithers {
    fn index_mut(&mut self, id: SlitherID) -> &mut Self::Output {
        self.data.get_mut(&id).unwrap()
    }
}
