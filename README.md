# FieldnameAccess Derive Macro

It is used to safely get values from the structure by field name when we
do not know exactly which field we will need at the moment but can match it and
do some actions based on other data

### Practical example

Let's say we have a User structure and Crit criteria for it. Having information on these two elements, we can determine our next steps.

```rust
use fieldname_access::FieldnameAccess;

#[derive(FieldnameAccess)]
struct User {
    name: String,
    age: u64,
    does_love_ranni: bool, // important 
}

struct Crit {
    value: String,
    field: String,
    kind: CritKind,
}

enum CritKind {
    Contains,
    Equals,
    BiggerThan,
}

let user = User {
    age: 2022,
    name: String::from("Radahn"),
    does_love_ranni: true,
};

let crits = vec![
    Crit {
        value: String::from("Ra"),
        field: String::from("name"),
        kind: CritKind::Contains,
    },
    Crit {
        value: String::from("true"),
        field: String::from("does_love_ranni"), // important
        kind: CritKind::Equals,
    },
    Crit {
        value: String::from("18"),
        field: String::from("age"),
        kind: CritKind::BiggerThan,
    },
];

let its_ok = crits.into_iter().all(|crit| {
    let user_field =
        user.field(&crit.field).expect("Criteria has wrong fieldname");
    match crit.kind {
        CritKind::Contains => match user_field {
            UserField::String(str) => str.contains(&crit.value),
            _ => panic!("Criteria has wrong value"),
        },
        CritKind::Equals => match user_field {
            UserField::String(str) => str.eq(&crit.value),
            UserField::U64(int) => int.eq(&crit.value.parse::<u64>().unwrap()),
            UserField::Bool(boolean) => {
                boolean.eq(&crit.value.parse::<bool>().unwrap())
            }
        },
        CritKind::BiggerThan => match user_field {
            UserField::U64(int) => int > &crit.value.parse::<u64>().unwrap(),
            _ => panic!("Criteria has wrong value"),
        },
    }
});
assert!(its_ok);
```

Also you can modify fields

```rust
if let Some(UserFieldMut::Bool(does_love_ranni)) =
    user.field_mut("does_love_ranni") // important
{
    *does_love_ranni = false; // HARAM
}
assert!(!user.does_love_ranni); //important
```
