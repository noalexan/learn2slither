use crate::action::Action;
use crate::state::State;

use std::{
    collections::HashMap,
    fs::File,
    io::{BufReader, BufWriter},
};

pub type QValue = f64;
pub type QRow = HashMap<Action, QValue>;
pub type QTable = HashMap<State, QRow>;

pub trait GetRow {
    fn row(&mut self, s: State) -> &mut QRow;
}

pub trait GetBest {
    fn best(&self) -> Action;
}

pub trait Savable {
    fn save(&mut self, path: &String) -> Result<(), Box<dyn std::error::Error>>;
}

pub trait Loadable {
    type Item;
    fn load(path: &String) -> Result<Self::Item, Box<dyn std::error::Error>>;
}

impl GetRow for QTable {
    fn row(&mut self, s: State) -> &mut QRow {
        self.entry(s).or_insert(HashMap::from([
            (Action::Forward, 0.0),
            (Action::Left, 0.0),
            (Action::Right, 0.0),
        ]))
    }
}

impl GetBest for QRow {
    fn best(&self) -> Action {
        *self
            .iter()
            .max_by(|&(_, q1), &(_, q2)| q1.total_cmp(q2))
            .unwrap()
            .0
    }
}

impl Savable for QTable {
    fn save(&mut self, path: &String) -> Result<(), Box<dyn std::error::Error>> {
        let file = BufWriter::new(File::create(path)?);
        serde_yaml::to_writer(file, self)?;
        Ok(())
    }
}

impl Loadable for QTable {
    type Item = Self;
    fn load(path: &String) -> Result<Self, Box<dyn std::error::Error>> {
        let file = BufReader::new(File::open(path)?);
        let data = serde_yaml::from_reader(file)?;
        Ok(data)
    }
}

pub fn q(q_table: &mut QTable, s: State, a: Action) -> &mut QValue {
    q_table.row(s).get_mut(&a).unwrap()
}
