use data::configdata::Sede;

/// Medal color of a placement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Red,
    Gold,
    Silver,
    Bronze,
    Green,
    Yellow,
}

/// Client-side rendering helpers for a [`Sede`].
pub trait SedeExt {
    /// The medal color for the given placement in this site.
    fn premio(&self, p: usize) -> Option<Color>;
}

impl SedeExt for Sede {
    fn premio(&self, p: usize) -> Option<Color> {
        if p <= self.entry.ouro {
            Some(Color::Gold)
        } else if p <= self.entry.prata {
            Some(Color::Silver)
        } else if p <= self.entry.bronze {
            Some(Color::Bronze)
        } else {
            None
        }
    }
}
