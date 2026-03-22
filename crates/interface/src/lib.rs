use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
};

pub trait Lossiness {}

#[derive(Debug, Clone)]
pub struct Lossy;

impl Lossiness for Lossy {}

#[derive(Debug, Clone)]
pub struct Lossless;

impl Lossiness for Lossless {}

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
    pub fn new(source: S, construct_target: Box<dyn FnOnce(S) -> T>, diff: Diff) -> Self {
        Self {
            source,
            construct_target,
            diff,
            _lossiness: PhantomData,
        }
    }

    pub fn diff(&self) -> &Diff {
        &self.diff
    }
}

impl<S, T> Translation<S, T, Lossless> {
    pub const fn is_lossy(&self) -> bool {
        false
    }

    pub fn translate(self) -> T {
        (self.construct_target)(self.source)
    }
}

impl<S, T> Translation<S, T, Lossy> {
    pub const fn is_lossy(&self) -> bool {
        true
    }

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

#[derive(Debug, Clone)]
pub struct Diff(String);

impl Diff {
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

    pub fn add<V: Debug>(self, name: &str, value: V) -> Self {
        const ADD: &str = "+";
        self.push(ADD, name, value)
    }

    pub fn sub<V: Debug>(self, name: &str, value: V) -> Self {
        const SUB: &str = "-";
        self.push(SUB, name, value)
    }
}

impl Display for Diff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub trait Upgrade<Next>
where
    Self: Sized,
{
    type Lossiness;

    fn upgrade(self) -> Translation<Self, Next, Self::Lossiness>;
}

pub trait Downgrade<Prev>
where
    Self: Sized,
{
    type Lossiness;

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
