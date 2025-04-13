use std::hash::Hasher;
use std::{
    borrow::Cow,
    fmt::{Display, Formatter},
    hash::Hash,
    ops::Deref,
    sync::Arc,
};

#[derive(Debug, Eq)]
pub enum ArcOrCowStr {
    Arc(Arc<str>),
    Cow(Cow<'static, str>),
}

impl Hash for ArcOrCowStr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        (**self).hash(state);
    }
}

impl PartialEq<ArcOrCowStr> for ArcOrCowStr {
    fn eq(&self, other: &ArcOrCowStr) -> bool {
        **self == **other
    }
}

impl From<Arc<str>> for ArcOrCowStr {
    fn from(v: Arc<str>) -> Self {
        Self::Arc(v)
    }
}

impl From<Cow<'static, str>> for ArcOrCowStr {
    fn from(v: Cow<'static, str>) -> Self {
        Self::Cow(v)
    }
}

impl From<&'static str> for ArcOrCowStr {
    fn from(v: &'static str) -> Self {
        Self::Cow(Cow::Borrowed(v))
    }
}

impl From<String> for ArcOrCowStr {
    fn from(v: String) -> Self {
        Self::Cow(Cow::Owned(v))
    }
}

impl AsRef<str> for ArcOrCowStr {
    fn as_ref(&self) -> &str {
        match self {
            Self::Arc(v) => v.as_ref(),
            Self::Cow(v) => v.as_ref(),
        }
    }
}

impl Deref for ArcOrCowStr {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl Display for ArcOrCowStr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&**self, f)
    }
}

#[cfg(test)]
mod test {
    mod arc_or_cow_str {
        use crate::util::ArcOrCowStr;
        use std::borrow::Cow;
        use std::sync::Arc;

        #[test]
        fn from_arc() {
            assert_eq!(
                ArcOrCowStr::from(Arc::from("hello world".to_string())),
                "hello world".into()
            );
        }

        #[test]
        fn from_cow() {
            assert_eq!(
                ArcOrCowStr::from(Cow::Borrowed("hello world")),
                "hello world".into()
            );
        }

        #[test]
        fn from_string() {
            assert_eq!(
                ArcOrCowStr::from("hello world".to_string()),
                "hello world".into()
            );
        }
    }
}
