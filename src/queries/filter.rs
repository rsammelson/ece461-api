use firestore::{
    select_filter_builder::FirestoreQueryFilterBuilder, FirestoreQueryFilter,
    FirestoreQueryFilterComposite, FirestoreQueryFilterCompositeOperator,
};
use semver::{Comparator, VersionReq};

pub fn versionreq_to_filter(
    q: &FirestoreQueryFilterBuilder,
    req: &VersionReq,
) -> Option<FirestoreQueryFilter> {
    let for_all_filters = req
        .comparators
        .iter()
        .map(|comp| comparator_to_filter(q, comp))
        .collect::<Option<Vec<_>>>()?;

    let operator = FirestoreQueryFilterCompositeOperator::And;

    Some(FirestoreQueryFilter::Composite(
        FirestoreQueryFilterComposite {
            for_all_filters,
            operator,
        },
    ))
}

fn comparator_to_filter(
    q: &FirestoreQueryFilterBuilder,
    comp: &Comparator,
) -> Option<FirestoreQueryFilter> {
    use semver::Op::*;
    match comp.op {
        //  =I.J.K — exactly the version I.J.K
        //  =I.J — equivalent to >=I.J.0, <I.(J+1).0
        //  =I — equivalent to >=I.0.0, <(I+1).0.0
        Exact => match comp {
            Comparator {
                major,
                minor: Some(minor),
                patch: Some(patch),
                ..
            } => q.field("Version").eq(format!("{major}.{minor}.{patch}")),
            Comparator {
                major,
                minor: Some(minor),
                patch: None,
                ..
            } => q.for_all([
                q.field("Version")
                    .greater_than_or_equal(format!("{major}.{minor}.0")),
                q.field("Version")
                    .less_than(format!("{major}.{}.0", minor + 1)),
            ]),
            Comparator {
                major,
                minor: None,
                patch: None,
                ..
            } => q.for_all([
                q.field("Version")
                    .greater_than_or_equal(format!("{major}.0.0")),
                q.field("Version").less_than(format!("{}.0.0", major + 1)),
            ]),
            _ => None,
        },
        //  >I.J.K
        //  >I.J — equivalent to >=I.(J+1).0
        //  >I — equivalent to >=(I+1).0.0
        Greater => match comp {
            Comparator {
                major,
                minor: Some(minor),
                patch: Some(patch),
                ..
            } => q
                .field("Version")
                .greater_than(format!("{major}.{minor}.{patch}")),
            Comparator {
                major,
                minor: Some(minor),
                patch: None,
                ..
            } => q
                .field("Version")
                .greater_than(format!("{major}.{minor}.0")),
            Comparator {
                major,
                minor: None,
                patch: None,
                ..
            } => q.field("Version").greater_than(format!("{major}.0.0")),
            _ => None,
        },
        //  >=I.J.K
        //  >=I.J — equivalent to >=I.J.0
        //  >=I — equivalent to >=I.0.0
        GreaterEq => match comp {
            Comparator {
                major,
                minor: Some(minor),
                patch: Some(patch),
                ..
            } => q
                .field("Version")
                .greater_than_or_equal(format!("{major}.{minor}.{patch}")),
            Comparator {
                major,
                minor: Some(minor),
                patch: None,
                ..
            } => q
                .field("Version")
                .greater_than_or_equal(format!("{major}.{minor}.0")),
            Comparator {
                major,
                minor: None,
                patch: None,
                ..
            } => q
                .field("Version")
                .greater_than_or_equal(format!("{major}.0.0")),
            _ => None,
        },
        //  <I.J.K
        //  <I.J — equivalent to <I.J.0
        //  <I — equivalent to <I.0.0
        Less => match comp {
            Comparator {
                major,
                minor: Some(minor),
                patch: Some(patch),
                ..
            } => q
                .field("Version")
                .less_than(format!("{major}.{minor}.{patch}")),
            Comparator {
                major,
                minor: Some(minor),
                patch: None,
                ..
            } => q.field("Version").less_than(format!("{major}.{minor}.0")),
            Comparator {
                major,
                minor: None,
                patch: None,
                ..
            } => q.field("Version").less_than(format!("{major}.0.0")),
            _ => None,
        },
        //  <=I.J.K
        //  <=I.J — equivalent to <I.(J+1).0
        //  <=I — equivalent to <(I+1).0.0
        LessEq => match comp {
            Comparator {
                major,
                minor: Some(minor),
                patch: Some(patch),
                ..
            } => q
                .field("Version")
                .less_than_or_equal(format!("{major}.{minor}.{patch}")),
            Comparator {
                major,
                minor: Some(minor),
                patch: None,
                ..
            } => q
                .field("Version")
                .less_than_or_equal(format!("{major}.{minor}.0")),
            Comparator {
                major,
                minor: None,
                patch: None,
                ..
            } => q
                .field("Version")
                .less_than_or_equal(format!("{major}.0.0")),
            _ => None,
        },
        //  ~I.J.K — equivalent to >=I.J.K, <I.(J+1).0
        //  ~I.J — equivalent to =I.J
        //  ~I — equivalent to =I
        Tilde => match comp {
            Comparator {
                major,
                minor: Some(minor),
                patch: Some(patch),
                ..
            } => q.for_all([
                q.field("Version")
                    .greater_than_or_equal(format!("{major}.{minor}.{patch}")),
                q.field("Version")
                    .less_than(format!("{major}.{}.0", minor + 1)),
            ]),
            Comparator {
                major,
                minor: Some(minor),
                patch: None,
                ..
            } => q.for_all([
                q.field("Version")
                    .greater_than_or_equal(format!("{major}.{minor}.0")),
                q.field("Version")
                    .less_than(format!("{major}.{}.0", minor + 1)),
            ]),
            Comparator {
                major,
                minor: None,
                patch: None,
                ..
            } => q.for_all([
                q.field("Version")
                    .greater_than_or_equal(format!("{major}.0.0")),
                q.field("Version").less_than(format!("{}.0.0", major + 1)),
            ]),
            _ => None,
        },
        //  ^I.J.K (for I>0) — equivalent to >=I.J.K, <(I+1).0.0
        //  ^0.J.K (for J>0) — equivalent to >=0.J.K, <0.(J+1).0
        //  ^0.0.K — equivalent to =0.0.K
        //  ^I.J (for I>0 or J>0) — equivalent to ^I.J.0
        //  ^0.0 — equivalent to =0.0
        //  ^I — equivalent to =I
        Caret => match comp {
            Comparator {
                major,
                minor: Some(minor),
                patch: Some(patch),
                ..
            } if *major > 0 => q.for_all([
                q.field("Version")
                    .greater_than_or_equal(format!("{major}.{minor}.{patch}")),
                q.field("Version").less_than(format!("{}.0.0", major + 1)),
            ]),
            Comparator {
                major: 0,
                minor: Some(minor),
                patch: Some(patch),
                ..
            } if minor > &0 => q.for_all([
                q.field("Version")
                    .greater_than_or_equal(format!("0.{minor}.{patch}")),
                q.field("Version").less_than(format!("0.{}.0", minor + 1)),
            ]),
            Comparator {
                major: 0,
                minor: Some(0),
                patch: Some(patch),
                ..
            } => q.for_all([q.field("Version").eq(format!("0.0.{patch}"))]),
            Comparator {
                major,
                minor: Some(minor),
                patch: None,
                ..
            } if *major > 0 || *minor > 0 => q.for_all([
                q.field("Version")
                    .greater_than_or_equal(format!("{major}.{minor}.0")),
                q.field("Version").less_than(format!("{}.0.0", major + 1)),
            ]),
            Comparator {
                major: 0,
                minor: Some(0),
                patch: None,
                ..
            } => q.for_all([
                q.field("Version").greater_than_or_equal("0.0.0"),
                q.field("Version").less_than("0.1.0"),
            ]),
            Comparator {
                major,
                minor: None,
                patch: None,
                ..
            } => q.for_all([
                q.field("Version")
                    .greater_than_or_equal(format!("{major}.0.0")),
                q.field("Version").less_than(format!("{}.0.0", major + 1)),
            ]),
            _ => None,
        },
        //  I.J.* — equivalent to =I.J
        //  I.* or I.*.* — equivalent to =I
        Wildcard => match comp {
            Comparator {
                major,
                minor: Some(minor),
                patch: None,
                ..
            } => q.for_all([
                q.field("Version")
                    .greater_than_or_equal(format!("{major}.{minor}.0")),
                q.field("Version")
                    .less_than(format!("{major}.{}.0", minor + 1)),
            ]),
            Comparator {
                major,
                minor: None,
                patch: None,
                ..
            } => q.for_all([
                q.field("Version")
                    .greater_than_or_equal(format!("{major}.0.0")),
                q.field("Version").less_than(format!("{}.0.0", major + 1)),
            ]),
            _ => None,
        },
        // non-exhaustive
        _ => None,
    }
}

pub fn comparator_requires_eq(comp: &Comparator) -> bool {
    use semver::Op::*;
    matches!(
        comp,
        Comparator {
            op: Exact,
            minor: Some(_),
            patch: Some(_),
            ..
        } | Comparator {
            op: Caret,
            major: 0,
            minor: Some(0),
            patch: Some(_),
            ..
        }
    )
}
