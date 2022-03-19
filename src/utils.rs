use std::{collections::HashSet, fmt::Debug, hash::Hash, ops::Add};

pub trait Check {
    fn check(self) -> CheckResult;
}

#[must_use]
#[derive(Debug)]
pub enum CheckResult {
    Success,
    Failure(Vec<String>),
}

impl CheckResult {
    pub fn compare<A, E>(expected: E, actual: A) -> Self
    where
        A: PartialEq<E> + Debug,
        E: Debug,
    {
        if actual == expected {
            Self::Success
        } else {
            Self::Failure(vec![format!("Expected '{expected:?}', got '{actual:?}'")])
        }
    }

    pub fn contains<C, T>(actual: C, expected: &[T]) -> Self
    where
        C: Contains<T>,
        T: PartialEq + Debug,
    {
        let failures = expected
            .iter()
            .filter_map(|e| {
                if actual.contains(e) {
                    None
                } else {
                    Some(format!("Missing '{e:?}'"))
                }
            })
            .collect::<Vec<_>>();

        failures.into()
    }

    pub fn any<T>(results: T) -> Self
    where
        T: Iterator<Item = Self>,
    {
        let failures = results
            .filter_map(|r| match r {
                Self::Success => None,
                Self::Failure(failures) => Some(failures),
            })
            .flatten()
            .collect::<Vec<_>>();

        failures.into()
    }

    pub fn missing(name: &str) -> Self {
        CheckResult::Failure(vec![format!("Missing {}", name)])
    }

    pub fn and(self, other: Self) -> Self {
        let mut self_failures: Vec<_> = self.into();
        let other_failures: Vec<_> = other.into();

        self_failures.extend(other_failures);
        self_failures.into()
    }

    pub fn as_bool(&self) -> bool {
        match *self {
            CheckResult::Success => true,
            CheckResult::Failure(_) => false,
        }
    }
}

impl Add for CheckResult {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        self.and(other)
    }
}

impl From<CheckResult> for Vec<String> {
    fn from(checkresult: CheckResult) -> Self {
        match checkresult {
            CheckResult::Success => Default::default(),
            CheckResult::Failure(failures) => failures,
        }
    }
}

impl From<Vec<String>> for CheckResult {
    fn from(failures: Vec<String>) -> Self {
        if failures.is_empty() {
            Self::Success
        } else {
            Self::Failure(failures)
        }
    }
}

pub trait Contains<T> {
    fn contains(&self, value: &T) -> bool;
}

impl<T> Contains<T> for HashSet<T>
where
    T: Hash + Eq,
{
    fn contains(&self, value: &T) -> bool {
        HashSet::contains(self, value)
    }
}

impl<T> Contains<T> for &[T]
where
    T: PartialEq,
{
    fn contains(&self, value: &T) -> bool {
        <[T]>::contains(self, value)
    }
}

macro_rules! check_option {
    ($s:ident, $t:ident) => {
        paste::paste! {
            if let Some($t) = $s.$t {
                $s.t.[<has_ $t>](&$t)
            } else {
                CheckResult::Success
            }
        }
    };
}

pub(crate) use check_option;
