// extract features from state

use blake3::Hasher;
use my_lib::my_tic_tac_toe::TicTacToeStatus;

use crate::{NextActionConstraint, UltTTT};

pub trait FeatureExtraction {
    fn extract_features(&self) -> Vec<f64>;
}

impl FeatureExtraction for UltTTT {
    fn extract_features(&self) -> Vec<f64> {
        let mut features: Vec<f64> = Vec::with_capacity(91);

        // values of 81 cells
        for (_, cell_value) in self
            .status_map
            .iter_map()
            .flat_map(|(c, _)| self.map.get_cell(c).iter_map())
        {
            let feature = match cell_value {
                TicTacToeStatus::Vacant => 0.0,
                TicTacToeStatus::First => 1.0,
                TicTacToeStatus::Second => -1.0,
                TicTacToeStatus::Tie => panic!("TicTacToeStatus::Tie in Mini Board not allowed"),
            };
            features.push(feature);
        }

        // Constraint: One-Hot over 10 fields (MiniBoard 0–8 + "free choice" at index 9)
        let mut constraint_encoding = [0.0; 10];
        let constraint_index = match self.next_action_constraint {
            NextActionConstraint::Init | NextActionConstraint::None => 9,
            NextActionConstraint::MiniBoard(board) => usize::from(board),
        };
        constraint_encoding[constraint_index] = 1.0;
        features.extend_from_slice(&constraint_encoding);

        features
    }
}

pub fn hash_features(features: &[f64]) -> i64 {
    let mut hasher = Hasher::new();

    for &f in features {
        // [u8; 8] – 8 Byte of f64
        let bytes = f.to_le_bytes();
        hasher.update(&bytes);
    }
    // final hash [u8; 32]
    let hash_bytes = hasher.finalize();
    // take first 8 Bytes
    let first_8 = &hash_bytes.as_bytes()[..8];
    // transform to i64
    i64::from_le_bytes(first_8.try_into().unwrap())
}
