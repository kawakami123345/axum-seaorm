use serde::de::DeserializeOwned;
use serde_json::{Value, json};

fn rejects_unversioned_or_identity_updates<T: DeserializeOwned>(value: Value) {
    assert!(serde_json::from_value::<T>(value.clone()).is_ok());
    let mut missing_revision = value.clone();
    missing_revision.as_object_mut().unwrap().remove("revision");
    assert!(serde_json::from_value::<T>(missing_revision).is_err());
    for field in ["id", "pub_id", "pubid"] {
        let mut changed_identity = value.clone();
        changed_identity[field] = json!("11111111-1111-4111-8111-111111111111");
        assert!(serde_json::from_value::<T>(changed_identity).is_err());
    }
}

#[test]
fn mutations_require_a_revision_and_reject_identity_fields() {
    rejects_unversioned_or_identity_updates::<crate::book::UpdateDto>(json!({
        "revision": 0,
        "title": "Book",
        "author": "Author",
        "publisher_id": "11111111-1111-4111-8111-111111111111",
        "price": 1000
    }));
    rejects_unversioned_or_identity_updates::<crate::book::ChangeAppliedAtDto>(json!({
        "revision": 0,
        "applied_at": null
    }));
    rejects_unversioned_or_identity_updates::<crate::publisher::UpdateDto>(json!({
        "revision": 0, "name": "Publisher"
    }));
    rejects_unversioned_or_identity_updates::<crate::shop::UpdateDto>(json!({
        "revision": 0, "name": "Shop"
    }));
}
