/// src/index.rs
use crate::attr::{Space, Vacuum};

/// Index define the properties of an index in second quantization.
/// name: The name of the index.
/// space: The space type of the index (General, Occupied, Virtual).
/// vacuum: The vacuum type of the index (Physical, Fermi, Bose).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Index {
    pub name: String,
    pub space: Space,
    pub vacuum: Vacuum,
}

impl Index {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            space: Space::General,
            vacuum: Vacuum::Physical,
        }
    }

    pub fn with_space(mut self, space: Space) -> Self {
        self.space = space;
        self
    }

    pub fn with_vacuum(mut self, vacuum: Vacuum) -> Self {
        self.vacuum = vacuum;
        self
    }

    pub fn build(self) -> Result<Self, String> {
        if self.space.is_allowed(self.vacuum) {
            Ok(self)
        } else {
            Err(format!(
                "Illegal Space {:?} for Vacuum {:?}",
                self.space, self.vacuum
            ))
        }
    }

    /// Some interface
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn space(&self) -> Space {
        self.space
    }
    pub fn vacuum(&self) -> Vacuum {
        self.vacuum
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_index() {
        let res = Index::new("i")
            .with_space(Space::Occupied)
            .with_vacuum(Vacuum::Fermi)
            .build();
        assert!(res.is_ok());

        let idx = res.unwrap();
        assert_eq!(idx.name, "i");
        assert_eq!(idx.space, Space::Occupied);
    }

    #[test]
    fn test_invalid_index() {
        let res = Index::new("p")
            .with_space(Space::Occupied)
            .with_vacuum(Vacuum::Physical)
            .build();
        assert!(res.is_err());
    }

    #[test]
    fn test_equality() {
        let a = Index::new("p");
        let b = Index::new("p");
        assert_eq!(a, b);
    }
}
