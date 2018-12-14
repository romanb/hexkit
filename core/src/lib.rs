// #![feature(existential_type)]
// #![feature(impl_trait_in_bindings)]

extern crate either;
extern crate nalgebra;
extern crate num_rational;
extern crate num_traits;
#[macro_use]
extern crate num_derive;
#[cfg(test)]
extern crate quickcheck;
#[cfg(test)]
extern crate rand;

pub mod geo;
pub mod grid;

