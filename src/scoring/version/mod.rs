#[cfg(test)]
mod tests;

use num_traits::{One, SaturatingAdd};
use semver::{Comparator, VersionReq};
use std::{
    collections::HashMap,
    ops::{Bound, RangeBounds},
};

#[derive(Clone, PartialEq, Eq, Debug)]
#[cfg_attr(test, derive(strum::EnumDiscriminants))]
#[cfg_attr(test, strum_discriminants(derive(strum::EnumIter)))]
enum PinStatus {
    Any,
    None,
    Pinned(u64),
    Within(Range<u64>),
    Less(u64),
    GreaterEq(u64),
}

#[derive(Clone, PartialEq, Eq, Debug)]
struct Range<T> {
    start: T,
    end: T,
}

impl<T> RangeBounds<T> for Range<T> {
    fn start_bound(&self) -> std::ops::Bound<&T> {
        Bound::Included(&self.start)
    }
    fn end_bound(&self) -> std::ops::Bound<&T> {
        Bound::Excluded(&self.end)
    }
}

impl<T> RangeBounds<T> for &Range<T> {
    fn start_bound(&self) -> std::ops::Bound<&T> {
        Bound::Included(&self.start)
    }
    fn end_bound(&self) -> std::ops::Bound<&T> {
        Bound::Excluded(&self.end)
    }
}

impl<T> Range<T> {
    fn intersect(self, other: impl RangeBounds<T>) -> Self
    where
        T: Ord + Copy + SaturatingAdd<Output = T> + One,
    {
        let start = match other.start_bound() {
            Bound::Included(other_start) => self.start.max(*other_start),
            Bound::Excluded(other_start) => self.start.max(other_start.saturating_add(&T::one())),
            Bound::Unbounded => self.start,
        };
        let end = match other.end_bound() {
            Bound::Included(other_end) => self.end.min(other_end.saturating_add(&T::one())),
            Bound::Excluded(other_end) => self.end.min(*other_end),
            Bound::Unbounded => self.end,
        };
        Range { start, end }
    }

    fn contains(&self, other: T) -> bool
    where
        T: PartialEq + PartialOrd + Copy,
    {
        self.start <= other && other < self.end
    }
}

impl PinStatus {
    fn update(self, other: Self) -> Self {
        let before = self.update_internal(other);
        use PinStatus::*;
        match before {
            Within(Range { start, end }) if start.saturating_add(1) == end => Self::Pinned(start),
            Within(Range { start, end }) if start >= end => Self::None,
            Less(e) if e == 1 => Self::Pinned(0),
            Less(e) if e == 0 => Self::None,
            b => b,
        }
    }

    fn update_internal(self, other: Self) -> Self {
        use PinStatus::*;
        match (self, other) {
            (None, _) | (_, None) => Self::None,
            (Any, o) => o,
            (s, Any) => s,

            (Pinned(s), Pinned(o)) if s == o => Self::Pinned(s),
            (Pinned(s), Within(o)) if o.contains(s) => Self::Pinned(s),
            (Pinned(s), Less(o)) if s < o => Self::Pinned(s),
            (Pinned(s), GreaterEq(o)) if s >= o => Self::Pinned(s),
            (Pinned(_), _) => Self::None,

            (Within(s), Within(o)) => Self::Within(s.intersect(o)),
            (Within(s), Less(o)) => Self::Within(s.intersect(..o)),
            (Within(s), GreaterEq(o)) => Self::Within(s.intersect(o..)),

            (Less(s), Less(o)) => Self::Less(s.min(o)),
            (Less(s), o) => o.update(Self::Less(s)),

            (GreaterEq(s), GreaterEq(o)) => Self::GreaterEq(s.max(o)),
            (GreaterEq(s), Less(o)) => Self::Within(Range { start: s, end: o }),
            (GreaterEq(s), o) => o.update(Self::GreaterEq(s)),

            (s, Pinned(o)) => Self::Pinned(o).update(s),
        }
    }
}

pub fn score_versionreq_pinned<T, U>(map: HashMap<T, U>) -> f64
where
    U: AsRef<str>,
{
    let (total, pinned) = map.into_values().fold((0, 0), |(total, pinned), ver| {
        (
            total + 1,
            match &ver.as_ref().parse() {
                Ok(v) if versionreq_pinned(v) => pinned + 1,
                _ => pinned,
            },
        )
    });

    if total == 0 {
        1.
    } else {
        (pinned as f64 / total as f64).min(1.).max(0.)
    }
}

fn versionreq_pinned(req: &VersionReq) -> bool {
    let major = req
        .comparators
        .iter()
        .fold(PinStatus::Any, |m, c| m.update(comparator_major_pinned(c)));

    if !matches!(major, PinStatus::Pinned(_)) {
        return false;
    }

    let minor = req
        .comparators
        .iter()
        .fold(PinStatus::Any, |m, c| m.update(comparator_minor_pinned(c)));

    if !matches!(minor, PinStatus::Pinned(_)) {
        return false;
    }

    true
}

fn comparator_major_pinned(comp: &Comparator) -> PinStatus {
    use semver::Op::*;
    match comp {
        Comparator {
            op: Exact | Tilde | Caret | Wildcard,
            major,
            ..
        } => PinStatus::Pinned(*major),
        Comparator {
            op: Greater,
            major,
            minor: Some(_),
            ..
        } => PinStatus::GreaterEq(*major),
        Comparator {
            op: Greater, major, ..
        } => PinStatus::GreaterEq((*major).saturating_add(1)),
        Comparator {
            op: GreaterEq,
            major,
            ..
        } => PinStatus::GreaterEq(*major),
        Comparator {
            op: Less | LessEq,
            major,
            minor: Some(_),
            ..
        } => PinStatus::Less((*major).saturating_add(1)),
        Comparator {
            op: Less, major, ..
        } => PinStatus::Less(*major),
        Comparator {
            op: LessEq, major, ..
        } => PinStatus::Less((*major).saturating_add(1)),
        // non exhaustive
        Comparator { .. } => PinStatus::Any,
    }
}

fn comparator_minor_pinned(comp: &Comparator) -> PinStatus {
    use semver::Op::*;
    match comp {
        Comparator {
            op: Exact | Tilde | Wildcard,
            minor: Some(minor),
            ..
        } => PinStatus::Pinned(*minor),
        Comparator {
            op: Greater,
            minor: Some(minor),
            patch: Some(_),
            ..
        } => PinStatus::GreaterEq(*minor),
        Comparator {
            op: Greater,
            minor: Some(minor),
            ..
        } => PinStatus::GreaterEq((*minor).saturating_add(1)),
        Comparator {
            op: GreaterEq,
            minor: Some(minor),
            ..
        } => PinStatus::GreaterEq(*minor),
        Comparator {
            op: Less,
            minor: Some(minor),
            patch: Some(_),
            ..
        } => PinStatus::Less((*minor).saturating_add(1)),
        Comparator {
            op: Less,
            minor: Some(minor),
            ..
        } => PinStatus::Less(*minor),
        Comparator {
            op: LessEq,
            minor: Some(minor),
            ..
        } => PinStatus::Less((*minor).saturating_add(1)),
        Comparator {
            op: Caret,
            major,
            minor: Some(minor),
            ..
        } if *major == 0 => PinStatus::Pinned(*minor),
        Comparator {
            op: Caret,
            minor: Some(minor),
            ..
        } => PinStatus::GreaterEq(*minor),
        Comparator { .. } => PinStatus::Any,
    }
}
