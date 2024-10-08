use std::fmt::Display;

use fieldname_access::FieldnameAccess;

#[derive(FieldnameAccess)]
struct TestStruct {
    name: String,
    age: u8,
    important: Option<ImportantInfo>,
}

struct ImportantInfo {
    does_love_ranni: bool, // important
}

#[test]
fn success_access() {
    let info = ImportantInfo {
        does_love_ranni: true, // important
    };
    let test_struct = TestStruct {
        age: 7,
        name: String::from("Radahn"),
        important: Some(info),
    };

    let age = match test_struct.field("age") {
        Some(TestStructField::U8(age)) => age,
        _ => panic!("Failed"),
    };
    assert_eq!(age, &test_struct.age);

    let name = match test_struct.field("name") {
        Some(TestStructField::String(name)) => name,
        _ => panic!("Failed"),
    };
    assert_eq!(name, &test_struct.name);

    let important = match test_struct.field("important") {
        Some(TestStructField::OptionImportantInfo(important)) => important,
        _ => panic!("Failed"),
    }
    .as_ref()
    .unwrap();

    assert_eq!(
        important.does_love_ranni,
        test_struct.important.as_ref().unwrap().does_love_ranni
    );
}

#[test]
fn success_access_mut() {
    let info = ImportantInfo {
        does_love_ranni: true, // important
    };
    let mut test_struct = TestStruct {
        age: 7,
        name: String::from("Radahn"),
        important: Some(info),
    };

    let age = match test_struct.field_mut("age") {
        Some(TestStructFieldMut::U8(age)) => age,
        _ => panic!("Failed"),
    };
    assert_eq!(age.clone(), test_struct.age);

    let name = match test_struct.field_mut("name") {
        Some(TestStructFieldMut::String(name)) => name,
        _ => panic!("Failed"),
    };
    assert_eq!(name.clone(), test_struct.name);

    let important = match test_struct.field_mut("important") {
        Some(TestStructFieldMut::OptionImportantInfo(important)) => important,
        _ => panic!("Failed"),
    }
    .as_ref()
    .unwrap();
    assert_eq!(
        important.does_love_ranni.clone(),
        test_struct.important.unwrap().does_love_ranni
    );
}

#[test]
fn failure_access() {
    let info = ImportantInfo {
        does_love_ranni: true, // important
    };
    let mut test_struct = TestStruct {
        age: 7,
        name: String::from("Radahn"),
        important: Some(info),
    };

    assert!(test_struct.field("not_important").is_none());
    assert!(test_struct.field_mut("not_really_important").is_none());
}

#[test]
fn field_mutation() {
    let info = ImportantInfo {
        does_love_ranni: true, // important
    };
    let mut test_struct = TestStruct {
        age: 7,
        name: String::from("Radahn"),
        important: Some(info),
    };

    let important = match test_struct.field_mut("important") {
        Some(TestStructFieldMut::OptionImportantInfo(important)) => important,
        _ => panic!("Failed"),
    };
    *important = None;

    assert!(test_struct.important.is_none());
}

#[test]
fn complex_test() {
    #[derive(FieldnameAccess)]
    struct User {
        name: String,
        age: u64,
        does_love_ranni: bool,
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

    let mut user = User {
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
            field: String::from("does_love_ranni"),
            kind: CritKind::Equals,
        },
        Crit {
            value: String::from("18"),
            field: String::from("age"),
            kind: CritKind::BiggerThan,
        },
    ];
    let its_ok = crits.into_iter().all(|crit| {
        let user_field = user
            .field(&crit.field)
            .expect("Criteria has wrong fieldname");
        match crit.kind {
            CritKind::Contains => match user_field {
                UserField::String(str) => str.contains(&crit.value),
                _ => panic!("Criteria has wrong value"),
            },
            CritKind::Equals => match user_field {
                UserField::String(str) => str.eq(&crit.value),
                UserField::U64(int) => int.eq(&crit.value.parse::<u64>().unwrap()),
                UserField::Bool(boolean) => boolean.eq(&crit.value.parse::<bool>().unwrap()),
            },
            CritKind::BiggerThan => match user_field {
                UserField::U64(int) => int > &crit.value.parse::<u64>().unwrap(),
                _ => panic!("Criteria has wrong value"),
            },
        }
    });
    assert!(its_ok);

    if let Some(UserFieldMut::Bool(does_love_ranni)) = user.field_mut("does_love_ranni") {
        *does_love_ranni = false;
    }
    assert!(!user.does_love_ranni);
}

#[derive(FieldnameAccess)]
struct TestComplexPath {
    name: std::option::Option<String>,
    age: std::option::Option<std::option::Option<i64>>,
}

#[test]
fn test_complex_type_path() {
    let structure = TestComplexPath {
        name: Some(String::from("Ranni")),
        age: Some(Some(321)),
    };

    if let Some(TestComplexPathField::OptionString(Some(val))) = structure.field("name") {
        assert_eq!(val, &"Ranni");
    } else {
        panic!("Failed");
    }

    if let Some(TestComplexPathField::OptionOptionI64(Some(Some(val)))) = structure.field("age") {
        assert_eq!(val, &321);
    } else {
        panic!("Failed");
    }
}

#[derive(FieldnameAccess, Clone, Copy)]
struct GenericStruct<'a, T, F>
where
    T: Display,
    F: Display,
{
    name: &'a T,
    age: F,
}

#[test]
fn generic_struct_fieldname_access() {
    let structure = GenericStruct {
        age: 123,
        name: &String::from("123"),
    };
    match structure.field("name").unwrap() {
        GenericStructField::T(name) => assert_eq!(*name, "123"),
        GenericStructField::F(_age) => panic!(),
    }
}

#[derive(FieldnameAccess)]
#[fieldname_enum(name = "Amazingly", derive = [Debug, Clone], derive_mut = [Debug])]
struct NamedFieldname {
    name: String,
    #[fieldname = "AmazingAge"]
    age: i64,
    dog_age: i64,
    cat_age: i64,
}

#[derive(FieldnameAccess)]
#[fieldname_enum(name = "AmazinglyTwo", derive_all = [Debug])]
#[allow(unused)]
struct NamedFieldnameTwo {
    name: String,
    #[fieldname = "AmazingAge"]
    age: i64,
}
#[test]
fn attributes() {
    let mut structure = NamedFieldname {
        age: 123,
        name: String::from("123"),
        dog_age: 123,
        cat_age: 123,
    };
    match structure.field("name").unwrap() {
        Amazingly::String(val) => {
            let val_clone = val.clone();
            assert_eq!(val_clone, "123")
        }
        Amazingly::AmazingAge(val) => assert_eq!(*val, 123),
        Amazingly::I64(val) => assert_eq!(*val, 123),
    }
    match structure.field_mut("name").unwrap() {
        AmazinglyMut::String(val) => {
            println!("{}", val);
            assert_eq!(val, "123")
        }
        AmazinglyMut::AmazingAge(val) => assert_eq!(*val, 123),
        AmazinglyMut::I64(val) => assert_eq!(*val, 123),
    }
}

#[test]
fn const_field_list() {
    assert_eq!(
        NamedFieldname::FIELDS,
        ["name", "age", "dog_age", "cat_age"]
    );
}

#[test]
fn simple_iter() {
    let structure = NamedFieldname {
        age: 123,
        cat_age: 123,
        dog_age: 123,
        name: String::from("boba"),
    };

    let mut res = Vec::new();
    structure.field_iter().for_each(|(name, val)| {
        res.push(format!("{}={}", name, val));
    });

    let expected = "name=boba\nage=123\ndog_age=123\ncat_age=123".to_string();
    assert_eq!(expected, res.join("\n"));
}

#[test]
fn generic_iter() {
    let external_var = String::from("boba");
    let structure = GenericStruct {
        age: 123,
        name: &external_var,
    };

    let mut res = Vec::new();
    structure.field_iter().for_each(|(name, val)| {
        res.push(format!("{}={}", name, val));
    });

    let expected = "name=boba\nage=123".to_string();
    assert_eq!(expected, res.join("\n"));
}

impl Display for Amazingly<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Amazingly::String(str) => write!(f, "{}", str),
            Amazingly::AmazingAge(age) => write!(f, "{}", age),
            Amazingly::I64(int) => write!(f, "{}", int),
        }
    }
}

impl<'a, T, F> Display for GenericStructField<'a, '_, T, F>
where
    T: Display,
    F: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GenericStructField::F(smth) => write!(f, "{}", smth),
            GenericStructField::T(smth) => write!(f, "{}", smth),
        }
    }
}
