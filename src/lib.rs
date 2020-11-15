pub mod data;
pub mod revelation;
pub mod config;
pub mod configdata;

#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
#[macro_use(quickcheck)]
extern crate quickcheck_macros;