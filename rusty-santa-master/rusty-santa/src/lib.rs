#[macro_use]
extern crate log;
extern crate rand;

use std::collections::{HashMap, HashSet};

use rand::{thread_rng, Rng};

#[derive(Clone)]
struct Matrix {
    keys: Vec<String>,
    indexes: HashMap<String, usize>,
    data: Vec<Vec<bool>>,
}

impl Matrix {
    pub fn new(keys: Vec<String>) -> Self {
        // Get size of matrix
        let size = keys.len();

        // Initialize indexes lookup map
        let mut indexes = HashMap::new();
        for (i, key) in keys.iter().enumerate() {
            indexes.insert(key.clone(), i);
        }

        // Initialize data vectors
        let mut data = vec![vec![true; size]; size];

        // Disallow giving gifts to oneself
        for i in 0..size {
            data[i][i] = false;
        }

        Matrix {
            keys: keys,
            indexes: indexes,
            data: data,
        }
    }

    pub fn get(&self, x: &str, y: &str) -> bool {
        let ix = self.indexes.get(x).unwrap();
        let iy = self.indexes.get(y).unwrap();
        self.data[*ix][*iy]
    }


    pub fn get_row(&self, x: &str) -> Vec<bool> {
        let ix = self.indexes.get(x).unwrap();
        self.data[*ix].clone()
    }


    pub fn set(&mut self, x: &str, y: &str, val: bool) {
        let ix = self.indexes.get(x).unwrap();
        let iy = self.indexes.get(y).unwrap();
        self.data[*ix][*iy] = val;
    }


    pub fn set_col(&mut self, y: &str, val: bool) {
        let iy = self.indexes.get(y).unwrap();
        for row in self.data.iter_mut() {
            row[*iy] = val;
        }
    }


    pub fn contains(&mut self, key: &str) -> bool {
        self.indexes.contains_key(key)
    }


    pub fn size(&self) -> usize {
        self.keys.len()
    }


    pub fn key_at(&self, index: usize) -> &str {
        &self.keys[index]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Constraint {
    ExcludePair {
        a: String,
        b: String,
    },
    Exclude {
        from: String,
        to: String,
    },
}


#[derive(Debug, Clone)]
pub struct Group {
    people_set: HashSet<String>,
    constraints: Vec<Constraint>,

    max_attempts: u32,
}

impl Group {

    pub fn new() -> Self {
        Group {
            people_set: HashSet::new(),
            constraints: vec![],
            max_attempts: 1000,
        }
    }


    pub fn add(&mut self, name: String) {
        self.people_set.insert(name);
    }

    fn add_constraint(&mut self, constraint: Constraint) {
        self.constraints.push(constraint);
    }

    pub fn exclude(&mut self, from: String, to: String) {
        let constraint = Constraint::Exclude { from: from, to: to };
        self.add_constraint(constraint);
    }


    pub fn exclude_pair(&mut self, a: String, b: String) {
        let constraint = Constraint::ExcludePair { a: a, b: b };
        self.add_constraint(constraint);
    }


    pub fn contains_name(&self, name: &str) -> bool {
        self.people_set.contains(name)
    }


    pub fn assign(&self) -> Result<Vec<(String, String)>, AssignError> {
        // Initialize the random number generator
        let mut rng = thread_rng();

        // Shuffle the people
        let mut people: Vec<String> = self.people_set.iter().cloned().collect();
        rng.shuffle(&mut people);

        'attempt: for _ in 0..self.max_attempts {

            // Initialize the gift possibility matrix
            let mut matrix = Matrix::new(people.clone());

            // Iterate over constraints, apply them to the matrix
            for constraint in self.constraints.iter() {
                match constraint {
                    &Constraint::ExcludePair{ ref a, ref b } => {
                        if !matrix.contains(a) {
                            return Err(AssignError::BadConstraint(format!("Unknown person \"{}\"", a)));
                        }
                        if !matrix.contains(b) {
                            return Err(AssignError::BadConstraint(format!("Unknown person \"{}\"", b)));
                        }
                        matrix.set(a, b, false);
                        matrix.set(b, a, false);
                    },
                    &Constraint::Exclude { ref from, ref to } => {
                        if !matrix.contains(from) {
                            return Err(AssignError::BadConstraint(format!("Unknown person \"{}\"", from)));
                        }
                        if !matrix.contains(to) {
                            return Err(AssignError::BadConstraint(format!("Unknown person \"{}\"", to)));
                        }
                        matrix.set(from, to, false);
                    }
                }
            };

            let mut assignments = vec![];
            for person in people.iter() {
                trace!("Drawing recipient for {}", person);

                // Get the possible names
                let mut basket = vec![];
                {
                    let row = matrix.get_row(&person);
                    for i in 0..row.len() {
                        if row[i] {
                            basket.push(matrix.key_at(i).to_owned());
                        }
                    }
                }
                trace!("Options: {:?}", basket);

                // Draw a random name
                if basket.is_empty() {
                    trace!("Attempt failed. Retrying...");
                    continue 'attempt;
                }
                let choice = rng.choose(&basket).unwrap();
                trace!("Picked {}!", choice);

                // Clear that person as a receiver from all rows
                matrix.set_col(choice, false);

                // Register assignment
                assignments.push((person.clone(), choice.clone()));
            }

            return Ok(assignments);
        }
        return Err(AssignError::GivingUp)
    }
}


#[derive(Debug)]
pub enum AssignError {
    BadConstraint(String),
    GivingUp,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matrix_init() {
        let keys = vec!["a".into(), "b".into(), "c".into()];
        let matrix = Matrix::new(keys);

        assert!(!matrix.get("a", "a"));
        assert!(!matrix.get("b", "b"));
        assert!(!matrix.get("c", "c"));

        assert!(matrix.get("a", "b"));
        assert!(matrix.get("a", "c"));
        assert!(matrix.get("b", "a"));
        assert!(matrix.get("b", "c"));
        assert!(matrix.get("c", "a"));
        assert!(matrix.get("c", "b"));
    }

    #[test]
    fn matrix_get_row() {
        let keys = vec!["a".into(), "b".into(), "c".into()];
        let mut matrix = Matrix::new(keys);
        assert_eq!(matrix.get_row("a"), vec![false, true, true]);
        assert_eq!(matrix.get_row("b"), vec![true, false, true]);
        assert_eq!(matrix.get_row("c"), vec![true, true, false]);
    }

    #[test]
    fn matrix_set() {
        let keys = vec!["a".into(), "b".into(), "c".into()];
        let mut matrix = Matrix::new(keys);

        assert!(matrix.get("a", "b"));
        matrix.set("a", "b", false);
        assert!(!matrix.get("a", "b"));
        matrix.set("a", "b", true);
        assert!(matrix.get("a", "b"));
    }

    #[test]
    fn matrix_contains() {
        let keys = vec!["a".into(), "b".into(), "c".into()];
        let mut matrix = Matrix::new(keys);

        assert!(matrix.contains("a"));
        assert!(matrix.contains("b"));
        assert!(matrix.contains("c"));
        assert!(!matrix.contains("d"));
        assert!(!matrix.contains("aa"));
    }

    #[test]
    fn matrix_size() {
        let keys = vec!["a".into(), "b".into(), "c".into()];
        let matrix = Matrix::new(keys);
        assert_eq!(3, matrix.size());

        let keys = vec!["a".into()];
        let matrix = Matrix::new(keys);
        assert_eq!(1, matrix.size());
    }


    #[test]
    fn group_simple() {
        let mut group = Group::new();

        group.add("a".into());
        group.add("b".into());
        group.add("c".into());

        let assignments = group.assign().unwrap();
        assert_eq!(assignments.len(), 3);

        for (from, to) in assignments {
            match from.as_ref() {
                "a" => assert!(to == "b" || to == "c"),
                "b" => assert!(to == "a" || to == "c"),
                "c" => assert!(to == "a" || to == "b"),
                _ => panic!(),
            }
        }
    }


    #[test]
    fn group_may_fail() {
        let mut group = Group::new();

        group.add("Sheldon".into());
        group.add("Amy".into());
        group.add("Leonard".into());
        group.add("Penny".into());
        group.add("Rajesh".into());

        group.exclude_pair("Sheldon".into(), "Amy".into());
        group.exclude_pair("Sheldon".into(), "Leonard".into());
        group.exclude_pair("Leonard".into(), "Penny".into());

        for i in 0..1000 {
            group.assign();
        }
    }
}
