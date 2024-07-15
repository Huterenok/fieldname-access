# FieldnameAccess Derive Macro

It is used to safely get values from the structure by field name when we
do not know exactly which field we will need at the moment but can match it and
do some actions based on other data

### Container attributes
* `#fieldname_enum(name = "NewName")` - Name of generated enum of possible values
```rust
use fieldname_access::FieldnameAccess;

#[derive(FieldnameAccess, Default)]
#[fieldname_enum(name = "NewName")]
struct NamedFieldname {
  name: String,
  age: i64,
}

let mut instance = NamedFieldname::default();
match instance.field("name").unwrap() {
    NewName::String(val) => {}
    NewName::I64(val) => {},
}
match instance.field_mut("name").unwrap() {
    NewNameMut::String(val) => {}
    NewNameMut::I64(val) => {},
}
```

* `#fieldname_enum(derive = [Debug, Clone], derive_mut = [Debug])` - Derive macroses for generated enums.
`derive` only for enum with immutable references, `derive_mut` only for enum with mutable references. 
It can be helpful when you want to derive `Clone` but only for immutable references as mutable are not clonable
```rust
use fieldname_access::FieldnameAccess;

#[derive(FieldnameAccess)]
#[fieldname_enum(derive = [Debug, Clone], derive_mut = [Debug])]
struct NamedFieldname {
  name: String,
  age: i64,
}
```

* `#fieldname_enum(derive_all = [Debug])` - Derive macroses for immutable and mutable generated enums
```rust
use fieldname_access::FieldnameAccess;

#[derive(FieldnameAccess)]
#[fieldname_enum(derive_all = [Debug])]
struct NamedFieldname {
  name: String,
  age: i64,
}
```

### Field attributes

* `#fieldname = "AmazingAge"` - Name of variant for field in generated enum.
It can be helpfull when you want to 'mark' field with specific variant name
```rust
use fieldname_access::FieldnameAccess;

#[derive(FieldnameAccess, Default)]
struct NamedFieldname {
  name: String,
  #[fieldname = "MyAge"]
  age: i64,
  dog_age: i64
}
let mut instance = NamedFieldname::default();
match instance.field("name").unwrap() {
    NamedFieldnameField::String(val) => {}
    NamedFieldnameField::MyAge(val) => {}
    NamedFieldnameField::I64(val) => {}
}
match instance.field_mut("name").unwrap() {
    NamedFieldnameFieldMut::String(val) => {}
    NamedFieldnameFieldMut::MyAge(val) => {}
    NamedFieldnameFieldMut::I64(val) => {}
}  
```

### Practical example

Let's say we have a User structure and Crit criteria for it. 

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
```

Based on this information `FieldnameAccess` will generate all possible types
of field and methods for `User` struct to access them by name:

```rust
enum UserField<'a> {
  String(&'a String),
  U64(&'a u64),
  Bool(&'a bool)
}

enum UserFieldMut<'a> {
  String(&'a mut String),
  U64(&'a mut u64),
  Bool(&'a mut bool)
}
```

Having information on these two elements, we can determine our next steps.

```rust
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
