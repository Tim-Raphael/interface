//! Typed, lossy-aware interface translation between API versions.
//!
//! This crate models the migration of data structures between versioned API
//! schemas. Each translation carries a [`Diff`] and is parameterised by a
//! [`Lossiness`] marker that distinguishes whether the conversion is
//! information-preserving ([`Lossless`]) or destructive ([`Lossy`]).
//!
//! # Quick start
//!
//! ```rust
//! use interface::{Diff, Lossless, Lossy, Translation, Upgrade, Downgrade};
//!
//! #[derive(Debug, PartialEq, Eq)]
//! struct UserV1 { name: String }
//!
//! #[derive(Debug, PartialEq, Eq)]
//! struct UserV2 { name: String, email: String }
//!
//! impl Upgrade<UserV2> for UserV1 {
//!     type Lossiness = Lossless;
//!     fn upgrade(self) -> Translation<Self, UserV2, Lossless> {
//!         let diff = Diff::new().add("email", "default@example.com");
//!         Translation::new(self, Box::new(|s| UserV2 {
//!             name: s.name,
//!             email: "default@example.com".into(),
//!         }), diff)
//!     }
//! }
//!
//! impl Downgrade<UserV1> for UserV2 {
//!     type Lossiness = Lossy;
//!     fn downgrade(self) -> Translation<Self, UserV1, Lossy> {
//!         let diff = Diff::new().sub("email", &self.email);
//!         Translation::new(self, Box::new(|s| UserV1 { name: s.name }), diff)
//!     }
//! }
//!
//! let t = UserV1 { name: "Alice".into() }.upgrade();
//! assert!(!t.is_lossy());
//! let v2 = t.translate();
//! assert_eq!(v2.email, "default@example.com");
//!
//! let t = v2.downgrade();
//! assert!(t.is_lossy());
//! let v1 = t.translate_lossy();
//! assert_eq!(v1.name, "Alice");
//! ```

use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
};

/// Sealed marker trait for translation lossiness.
///
/// Implemented only by [`Lossy`] and [`Lossless`].
pub trait Lossiness {}

/// Marker: the translation drops information present in the source.
#[derive(Debug, Clone)]
pub struct Lossy;

impl Lossiness for Lossy {}

/// Marker: the translation preserves all information from the source.
#[derive(Debug, Clone)]
pub struct Lossless;

impl Lossiness for Lossless {}

/// A pending translation from `Source` to `Target`.
///
/// The conversion is deferred: the source value and a constructor closure are
/// held together until the caller explicitly calls [`translate`] (lossless) or
/// [`translate_lossy`] (lossy). This lets callers inspect the [`Diff`] and
/// decide whether to proceed.
///
/// [`translate`]: Translation::<_, _, Lossless>::translate
/// [`translate_lossy`]: Translation::<_, _, Lossy>::translate_lossy
pub struct Translation<Source, Target, Lossiness> {
    source: Source,
    construct_target: Box<dyn FnOnce(Source) -> Target>,
    diff: Diff,
    _lossiness: PhantomData<Lossiness>,
}

impl<S, T, L> Translation<S, T, L>
where
    L: Lossiness,
{
    /// Build a translation from its constituent parts.
    ///
    /// Normally called from within [`Upgrade::upgrade`] or
    /// [`Downgrade::downgrade`] implementations.
    pub fn new(source: S, construct_target: Box<dyn FnOnce(S) -> T>, diff: Diff) -> Self {
        Self {
            source,
            construct_target,
            diff,
            _lossiness: PhantomData,
        }
    }

    /// Returns the structural diff between source and target schemas.
    pub fn diff(&self) -> &Diff {
        &self.diff
    }
}

impl<S, T> Translation<S, T, Lossless> {
    /// Always returns `false`; present for API symmetry with the [`Lossy`] impl.
    pub const fn is_lossy(&self) -> bool {
        false
    }

    /// Consume the translation and produce the target value.
    pub fn translate(self) -> T {
        (self.construct_target)(self.source)
    }
}

impl<S, T> Translation<S, T, Lossy> {
    /// Always returns `true`; signals that calling [`translate_lossy`] will
    /// drop data.
    ///
    /// [`translate_lossy`]: Translation::<_, _, Lossy>::translate_lossy
    pub const fn is_lossy(&self) -> bool {
        true
    }

    /// Consume the translation and produce the target value, accepting data loss.
    pub fn translate_lossy(self) -> T {
        (self.construct_target)(self.source)
    }
}

impl<S, T> Display for Translation<S, T, Lossy>
where
    T: Debug,
    S: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "The translation was lossy:\n{}", self.diff())
    }
}

impl<S, T> Display for Translation<S, T, Lossless>
where
    T: Debug,
    S: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "The translation was lossless:\n{}", self.diff())
    }
}

/// A human-readable record of fields added or removed during a translation.
///
/// Built with a fluent API:
///
/// ```rust
/// use interface::Diff;
///
/// let diff = Diff::new()
///     .add("email", "default@example.com")
///     .sub("legacy_id", 42u32);
///
/// println!("{diff}");
/// ```
#[derive(Debug, Clone)]
pub struct Diff(String);

impl Diff {
    /// Create an empty diff.
    pub fn new() -> Self {
        Self(String::new())
    }

    fn push<V: Debug>(mut self, sign: &str, name: &str, value: V) -> Self {
        let formatted = format!("{:#?}", value);
        let prefixed = formatted
            .lines()
            .enumerate()
            .map(|(i, line)| {
                if i == 0 {
                    format!("{line}")
                } else {
                    format!("{sign}{line}")
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        self.0.push_str(&format!("{sign}{name}: {prefixed}\n"));

        self
    }

    /// Record a field that exists in the target but not the source.
    pub fn add<V: Debug>(self, name: &str, value: V) -> Self {
        const ADD: &str = "+";
        self.push(ADD, name, value)
    }

    /// Record a field that exists in the source but not the target.
    pub fn sub<V: Debug>(self, name: &str, value: V) -> Self {
        const SUB: &str = "-";
        self.push(SUB, name, value)
    }
}

impl Default for Diff {
    fn default() -> Self {
        Self::new()
    }
}

impl Display for Diff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Implement this to describe how `Self` migrates forward to `Next`.
///
/// The associated [`Lossiness`] type must be either [`Lossy`] or [`Lossless`],
/// encoding at the type level whether the upgrade is destructive.
pub trait Upgrade<Next>
where
    Self: Sized,
{
    /// [`Lossy`] if the upgrade drops fields; [`Lossless`] otherwise.
    type Lossiness;

    /// Produce a [`Translation`] without executing it.
    fn upgrade(self) -> Translation<Self, Next, Self::Lossiness>;
}

/// Implement this to describe how `Self` migrates backward to `Prev`.
///
/// Downgrades are almost always [`Lossy`] because older schemas typically
/// have fewer fields.
pub trait Downgrade<Prev>
where
    Self: Sized,
{
    /// [`Lossy`] if the downgrade drops fields; [`Lossless`] otherwise.
    type Lossiness;

    /// Produce a [`Translation`] without executing it.
    fn downgrade(self) -> Translation<Self, Prev, Self::Lossiness>;
}

#[cfg(test)]
mod tests {
    use super::*;

    mod v1 {
        use super::*;

        #[derive(Debug, PartialEq, Eq)]
        pub struct User {
            name: String,
        }

        impl User {
            pub fn new(name: String) -> Self {
                Self { name }
            }
        }

        impl<'a> Upgrade<v2::User> for User {
            type Lossiness = Lossless;

            fn upgrade(self) -> Translation<Self, v2::User, Self::Lossiness> {
                let diff = Diff::new().add("email", v2::Email::default());

                let construct_target = Box::from(|s: Self| -> v2::User {
                    v2::User::new(v2::Name::from(s.name), v2::Email::default())
                });

                Translation::new(self, construct_target, diff)
            }
        }
    }

    mod v2 {
        use super::*;

        #[derive(Debug, PartialEq, Eq)]
        pub struct Name(String);

        impl From<String> for Name {
            fn from(value: String) -> Self {
                Self(value)
            }
        }

        #[derive(Debug, PartialEq, Eq)]
        pub struct Email(String);

        impl From<String> for Email {
            fn from(value: String) -> Self {
                Self(value)
            }
        }

        impl Default for Email {
            fn default() -> Self {
                Self("mail@example.com".to_string())
            }
        }

        #[derive(Debug, PartialEq, Eq)]
        pub struct User {
            name: Name,
            email: Email,
        }

        impl User {
            pub fn new(name: Name, email: Email) -> Self {
                Self { name, email }
            }
        }

        impl<'a> Downgrade<v1::User> for User {
            type Lossiness = Lossy;

            fn downgrade(self) -> Translation<Self, v1::User, Self::Lossiness> {
                let diff = Diff::new().sub("email", &self.email);

                let construct_target = Box::from(|s: Self| -> v1::User { v1::User::new(s.name.0) });

                Translation::new(self, construct_target, diff)
            }
        }
    }

    #[test]
    fn upgrade_from_v1_to_v2() {
        let user = v1::User::new("Foo".to_string());
        let translation = user.upgrade();

        assert!(!translation.is_lossy());
        assert_eq!(
            translation.translate(),
            v2::User::new(v2::Name::from("Foo".to_string()), v2::Email::default())
        );
    }

    #[test]
    fn downgrade_from_v2_to_v1() {
        let user = v2::User::new(
            v2::Name::from("Foo".to_string()),
            v2::Email::from("Bar".to_string()),
        );
        let translation = user.downgrade();

        assert!(translation.is_lossy());
        assert_eq!(
            translation.translate_lossy(),
            v1::User::new("Foo".to_string())
        );
    }
}
