use serde::de::DeserializeOwned;
use serde_json::{Value, json};

pub(crate) fn annual_totals(books: Vec<book::Book>) -> Vec<book::AnnualTotals> {
    let mut years = std::collections::BTreeMap::new();
    for book in books {
        if let Some(at) = book.applied_at() {
            let year = at.to_offset(time::UtcOffset::UTC).year();
            let totals = years.entry(year).or_insert(book::AnnualTotals {
                year,
                count: 0,
                total_amount: 0,
            });
            totals.count += 1;
            totals.total_amount += i64::from(book.price());
        }
    }
    years.into_values().collect()
}

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
