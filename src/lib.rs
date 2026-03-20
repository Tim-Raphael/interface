use std::{
    fmt::{Debug, Display},
    marker::PhantomData,
};

pub trait Relation {}

#[derive(Debug, Clone)]
pub struct Lossy;

impl Relation for Lossy {}

#[derive(Debug, Clone)]
pub struct Lossless;

impl Relation for Lossless {}

#[derive(Debug, Clone)]
pub struct Translation<Source, Target, Relation> {
    source: Source,
    target: Target,
    _relation: PhantomData<Relation>,
}

impl<S, T, R> Translation<S, T, R>
where
    R: Relation,
{
    pub fn new(source: S, target: T) -> Self {
        Self {
            source,
            target,
            _relation: PhantomData,
        }
    }

    pub fn source(&self) -> &S {
        &self.source
    }

    pub fn target(&self) -> &T {
        &self.target
    }

    fn diff(&self) -> Diff<'_, S, T> {
        self.into()
    }
}

impl<S, T> Display for Translation<S, T, Lossy>
where
    T: Debug,
    S: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "The translation was lossy:\n\n{}", self.diff())
    }
}

impl<S, T> Display for Translation<S, T, Lossless>
where
    T: Debug,
    S: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "The translation was lossless:\n\n{}", self.diff())
    }
}

#[derive(Debug, Clone)]
pub struct Diff<'a, Left, Right> {
    left: &'a Left,
    right: &'a Right,
}

impl<'a, S, T, R> From<&'a Translation<S, T, R>> for Diff<'a, S, T>
where
    R: Relation,
{
    fn from(value: &'a Translation<S, T, R>) -> Self {
        Self {
            left: value.source(),
            right: value.target(),
        }
    }
}

impl<L, R> Display for Diff<'_, L, R>
where
    L: Debug,
    R: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}\n{:#?}", self.left, self.right)
    }
}

pub trait Upgrade<Next>
where
    Self: Sized,
{
    type Relation;

    fn upgrade(self) -> Translation<Self, Next, Self::Relation>;
}

pub trait Downgrade<Prev>
where
    Self: Sized,
{
    type Relation;

    fn downgrade(self) -> Translation<Self, Prev, Self::Relation>;
}

#[cfg(test)]
mod tests {
    use super::*;

    mod v1 {
        use super::*;

        #[derive(Debug)]
        pub struct User {
            name: String,
        }

        impl User {
            pub fn new(name: String) -> Self {
                Self { name }
            }
        }

        impl Upgrade<v2::User> for User {
            type Relation = Lossless;

            fn upgrade(self) -> Translation<Self, v2::User, Self::Relation> {
                let name = v2::Name::from(self.name.clone());
                let email = v2::Email::default();
                let v2_user = v2::User::new(name, email);
                Translation::new(self, v2_user)
            }
        }
    }

    mod v2 {
        use super::*;

        #[derive(Debug)]
        pub struct Name(String);

        impl From<String> for Name {
            fn from(value: String) -> Self {
                Self(value)
            }
        }

        #[derive(Debug)]
        pub struct Email(String);

        impl Default for Email {
            fn default() -> Self {
                Self("mail@example.com".to_string())
            }
        }

        #[derive(Debug)]
        pub struct User {
            name: Name,
            email: Email,
        }

        impl User {
            pub fn new(name: Name, email: Email) -> Self {
                Self { name, email }
            }
        }
    }

    #[test]
    fn upgrade_from_v1_to_v2() {
        let user = v1::User::new("Foo".to_string());
        println!("{}", user.upgrade());
    }
}
