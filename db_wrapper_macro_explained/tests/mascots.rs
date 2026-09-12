use db_wrapper_macro_explained::serialization::Convert;
use db_wrapper_macro_explained::actual_macro_usage::plain_mascot::PlainMascot;
use db_wrapper_macro_explained::actual_macro_usage::serializer_mascot::SerializerMascot;
use db_wrapper_macro_explained::payload::*;

#[test]
fn plain_get_thing() {
    let m = PlainMascot { conn: "alpha".into() };
    let out = m.get_thing(GetThingIn { id: 7 }).unwrap();
    assert_eq!(out.value, "alpha says thing #7");
}

#[test]
fn plain_get_combined() {
    let m = PlainMascot { conn: "alpha".into() };
    let out = m.get_combined(GetCombinedIn { id: 3 }).unwrap();
    assert_eq!(out.value, "alpha says thing #3");
    assert_eq!(out.other, 5);
}

#[test]
fn serializer_get_thing() {
    let m = SerializerMascot { conn: "beta".into() };
    let bytes = GetThingIn { id: 9 }.to_payload();
    let out_bytes = m.get_thing(bytes);
    let out = GetThingOut::un_payloadify(&out_bytes).unwrap();
    assert_eq!(out.value, "beta says thing #9");
}

#[test]
fn serializer_get_combined() {
    let m = SerializerMascot { conn: "beta".into() };
    let bytes = GetCombinedIn { id: 4 }.to_payload();
    let out_bytes = m.get_combined(bytes);
    let out = CombinedOut::un_payloadify(&out_bytes).unwrap();
    assert_eq!(out.value, "beta says thing #4");
    assert_eq!(out.other, 4);
}
