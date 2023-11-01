use katsuba_utils::hash::*;

#[test]
fn test_djb2() {
    assert_eq!(djb2(b"m_packedName"), 307420154);
}

#[test]
fn test_string_id() {
    assert_eq!(string_id(b"std::string"), 1497788074);
    assert_eq!(string_id(b"class FishTournamentEntry"), 1725212200);
    assert_eq!(
        string_id(b"class NonCombatMayCastSpellTemplate*"),
        920052956
    );
}
