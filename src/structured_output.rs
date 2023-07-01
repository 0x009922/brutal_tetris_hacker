use crate::algorithm::PlacementResult;
use crate::tetra::{Tetra as BaseTetra, TETRAS};
use crate::util::Pos;
use serde::Serialize;
use std::collections::{BTreeMap, BTreeSet, HashMap};

#[derive(Debug, Serialize)]
pub struct Output {
    tetras: BTreeMap<usize, Tetra>,
    placements: BTreeSet<Placement>,
}

impl Output {
    pub fn new(placements: &BTreeSet<PlacementResult>) -> Self {
        let tetras_reverse: HashMap<&'static BaseTetra, usize> =
            TETRAS.iter().enumerate().map(|(a, b)| (b, a)).collect();

        let placements = placements
            .iter()
            .map(|placement| Placement {
                free: placement.free,
                tetras: placement
                    .placement
                    .iter()
                    .map(|tetra_pos| {
                        let tetra_id = *tetras_reverse
                            .get(tetra_pos.tetra)
                            .expect("All tetras should be presented in the map");
                        TetraPos {
                            tetra: tetra_id,
                            pos: tetra_pos.position,
                        }
                    })
                    .collect(),
            })
            .collect();

        let tetras = tetras_reverse
            .into_iter()
            .map(|(tetra, id)| {
                let tetra: Tetra = tetra.into();
                (id, tetra)
            })
            .collect();

        Self { placements, tetras }
    }
}

#[derive(Debug, Serialize)]
pub struct Tetra {
    positions: Vec<Pos>,
}

impl From<&'_ BaseTetra> for Tetra {
    fn from(value: &BaseTetra) -> Self {
        Self {
            positions: value.iter().map(|x| *x).collect(),
        }
    }
}

#[derive(Debug, Serialize, Ord, PartialOrd, Eq, PartialEq)]
pub struct Placement {
    tetras: BTreeSet<TetraPos>,
    free: usize,
}

#[derive(Debug, Serialize, Ord, PartialOrd, Eq, PartialEq)]
pub struct TetraPos {
    tetra: usize,
    pos: Pos,
}
