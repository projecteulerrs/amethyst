//! # amethyst_locale
//!
//! Localisation binding a `Fluent` file to an Asset<Locale> via the use of amethyst_assets.

#![warn(
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    rust_2018_compatibility
)]
#![warn(clippy::all)]
#![allow(clippy::new_without_default)]

use amethyst_assets::{Asset, Format, Handle};
use amethyst_core::ecs::prelude::VecStorage;
use amethyst_error::Error;
pub use fluent::*;
use serde::{Deserialize, Serialize};

/// Loads the strings from localisation files.
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct LocaleFormat;

amethyst_assets::register_format_type!(Locale);

amethyst_assets::register_format!("FTL", LocaleFormat as Locale);
impl Format<Locale> for LocaleFormat {
    fn name(&self) -> &'static str {
        "FTL"
    }

    fn import_simple(&self, bytes: Vec<u8>) -> Result<Locale, Error> {
        let s = String::from_utf8(bytes)?;

        let resource = FluentResource::try_new(s).expect("Failed to parse locale data");

        Ok(Locale { resource })
    }
}

/// A handle to a locale.
pub type LocaleHandle = Handle<Locale>;

/// A loaded locale.
#[allow(missing_debug_implementations)]
pub struct Locale {
    /// The backing fluent resource.
    pub resource: FluentResource,
}

impl Asset for Locale {
    const NAME: &'static str = "locale::Locale";
    type Data = Locale;
    type HandleStorage = VecStorage<LocaleHandle>;
}
