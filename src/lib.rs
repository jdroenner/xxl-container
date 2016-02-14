
extern crate byteorder;
extern crate bincode;

#[cfg(feature = "rustc_serialize")]
extern crate rustc_serialize;

#[cfg(feature = "serde")]
extern crate serde;

pub mod container;
pub mod error;
pub mod io;
pub mod mem {
    pub mod veccontainer;
}
pub mod adapter {
    pub mod converter;

    #[cfg(feature = "rustc_serialize")]
    pub mod serialize;

    #[cfg(feature = "serde")]
    pub mod serde;
}
