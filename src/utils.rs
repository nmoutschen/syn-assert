pub trait Check {
    fn check(self) -> bool;
}

macro_rules! option_check {
    ($s:ident, $t:ident) => {
        paste::paste! {
            if let Some($t) = $s.$t {
                if !$s.t.[<has_ $t>](&$t) {
                    return false;
                }
            }
        }
    };
}

macro_rules! vec_check {
    ($s:ident, $t:ident) => {
        paste::paste! {
            if !$s.t.[<has_ $t>](&$s.$t) {
                return false;
            }
        }
    };
}

pub(crate) use option_check;
pub(crate) use vec_check;
