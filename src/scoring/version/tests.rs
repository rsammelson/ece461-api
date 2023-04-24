use super::*;
use quickcheck_macros::quickcheck;
use strum::IntoEnumIterator;

#[test]
fn versionreq_pinned_test() {
    // minor version not pinned
    assert!(!versionreq_pinned(&VersionReq::parse("1.2").unwrap()));

    assert!(versionreq_pinned(&VersionReq::parse("0.2.3").unwrap()));

    assert!(versionreq_pinned(&VersionReq::parse("1.2.*").unwrap()));

    assert!(versionreq_pinned(
        &VersionReq::parse("<1.2.5,>=1.2").unwrap()
    ));

    // minor unpinned
    assert!(!versionreq_pinned(
        &VersionReq::parse("<1.2.5,>=1").unwrap()
    ));

    assert!(versionreq_pinned(
        &VersionReq::parse("<1.2.5,=0.2").unwrap()
    ));
}

#[test]
fn versionreq_pinned_map() {
    let empty: HashMap<(), &str> = HashMap::new();
    assert_eq!(score_versionreq_pinned(empty), 1.);

    let one_of_one = HashMap::from([(1, "0.1.2")]);
    assert_eq!(score_versionreq_pinned(one_of_one), 1.);

    let zero_of_one = HashMap::from([(1, ">2")]);
    assert_eq!(score_versionreq_pinned(zero_of_one), 0.);

    let one_of_two = HashMap::from([(1, "0.1.2"), (2, "^1")]);
    assert_eq!(score_versionreq_pinned(one_of_two), 1. / 2.);

    let two_of_three = HashMap::from([(1, "=1.2.3"), (2, "^1"), (3, "<1.3,>=1.2")]);
    assert_eq!(score_versionreq_pinned(two_of_three), 2. / 3.);

    let bad_parse = HashMap::from([(1, "url")]);
    assert_eq!(score_versionreq_pinned(bad_parse), 0.);
}

impl quickcheck::Arbitrary for PinStatus {
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        let disc = g
            .choose(&PinStatusDiscriminants::iter().collect::<Vec<_>>())
            .expect("choose value")
            .clone();
        match disc {
            PinStatusDiscriminants::Any => PinStatus::Any,
            PinStatusDiscriminants::None => PinStatus::None,
            PinStatusDiscriminants::Pinned => PinStatus::Pinned(u64::arbitrary(g)),
            PinStatusDiscriminants::Within => PinStatus::Within(Range {
                start: u64::arbitrary(g),
                end: u64::arbitrary(g),
            }),
            PinStatusDiscriminants::Less => PinStatus::Less(u64::arbitrary(g)),
            PinStatusDiscriminants::GreaterEq => PinStatus::GreaterEq(u64::arbitrary(g)),
        }
    }
}

#[quickcheck]
fn pinstatus_update_commutative(first: PinStatus, second: PinStatus) -> bool {
    first.clone().update(second.clone()) == second.update(first)
}

#[test]
fn pinstatus_none() {
    assert_eq!(PinStatus::None.update(PinStatus::Any), PinStatus::None);
    assert_eq!(PinStatus::None.update(PinStatus::None), PinStatus::None);
    assert_eq!(
        PinStatus::None.update(PinStatus::Pinned(3)),
        PinStatus::None
    );
    assert_eq!(PinStatus::None.update(PinStatus::Less(3)), PinStatus::None);
    assert_eq!(
        PinStatus::None.update(PinStatus::GreaterEq(3)),
        PinStatus::None
    );
    assert_eq!(
        PinStatus::None.update(PinStatus::Within(Range { start: 1, end: 3 })),
        PinStatus::None
    );
}

#[test]
fn pinstatus_any() {
    assert_eq!(PinStatus::Any.update(PinStatus::Any), PinStatus::Any);
    assert_eq!(
        PinStatus::Any.update(PinStatus::Pinned(3)),
        PinStatus::Pinned(3),
    );
    assert_eq!(
        PinStatus::Any.update(PinStatus::Less(3)),
        PinStatus::Less(3),
    );
    assert_eq!(
        PinStatus::Any.update(PinStatus::GreaterEq(3)),
        PinStatus::GreaterEq(3),
    );
    assert_eq!(
        PinStatus::Any.update(PinStatus::Within(Range { start: 1, end: 3 })),
        PinStatus::Within(Range { start: 1, end: 3 }),
    );
}

#[test]
fn pinstatus_pinned() {
    let num = 17;
    let orig = || PinStatus::Pinned(num);
    assert_eq!(orig().update(PinStatus::Pinned(num + 1)), PinStatus::None);
    assert_eq!(orig().update(orig()), orig());
    assert_eq!(orig().update(PinStatus::Less(num + 4)), orig());
    assert_eq!(orig().update(PinStatus::Less(num - 4)), PinStatus::None);
    assert_eq!(
        orig().update(PinStatus::GreaterEq(num + 4)),
        PinStatus::None
    );
    assert_eq!(
        orig().update(PinStatus::Within(Range {
            start: num - 2,
            end: num + 3
        })),
        orig()
    );
    assert_eq!(
        orig().update(PinStatus::Within(Range {
            start: num - 4,
            end: num - 3
        })),
        PinStatus::None,
    );
}

#[test]
fn pinstatus_within() {
    let start = 17;
    let end = 20;
    let orig = || PinStatus::Within(Range { start, end });
    assert_eq!(
        orig().update(PinStatus::Pinned(start)),
        PinStatus::Pinned(start)
    );
    assert_eq!(
        orig().update(PinStatus::Pinned(start + 2)),
        PinStatus::Pinned(start + 2)
    );
    assert_eq!(orig().update(PinStatus::Pinned(start - 2)), PinStatus::None);
    assert_eq!(orig().update(orig()), orig());
    assert_eq!(orig().update(PinStatus::Less(end + 4)), orig());
    assert_eq!(
        orig().update(PinStatus::Less(end - 1)),
        PinStatus::Within(Range {
            start,
            end: end - 1
        })
    );
    assert_eq!(orig().update(PinStatus::Less(start)), PinStatus::None);
    assert_eq!(orig().update(PinStatus::GreaterEq(start)), orig());
    assert_eq!(orig().update(PinStatus::GreaterEq(end)), PinStatus::None);
    assert_eq!(
        orig().update(PinStatus::Within(Range {
            start: start - 2,
            end: end + 3
        })),
        orig()
    );
    assert_eq!(
        orig().update(PinStatus::Within(Range {
            start: start - 4,
            end: end - 3
        })),
        PinStatus::None,
    );
    assert_eq!(
        orig().update(PinStatus::Within(Range {
            start: start + 1,
            end: end + 2
        })),
        PinStatus::Within(Range {
            start: start + 1,
            end
        })
    );
}

#[test]
fn pinstatus_consolidate() {
    assert_eq!(
        PinStatus::Within(Range { start: 4, end: 5 }).update(PinStatus::Any),
        PinStatus::Pinned(4)
    );
    assert_eq!(
        PinStatus::Within(Range { start: 4, end: 4 }).update(PinStatus::Any),
        PinStatus::None
    );
    assert_eq!(
        PinStatus::Within(Range { start: 4, end: 3 }).update(PinStatus::Any),
        PinStatus::None
    );
}
