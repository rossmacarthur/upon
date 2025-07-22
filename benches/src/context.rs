use rand::Rng;

#[derive(serde::Serialize)]
pub struct Users {
    pub title: String,
    pub users: Vec<User>,
}

#[derive(serde::Serialize)]
pub struct User {
    pub name: String,
    pub age: u32,
    pub is_disabled: bool,
}

pub fn random(n: usize) -> Users {
    let mut rng = rand::thread_rng();
    let title = (0..20).map(|_| rng.gen_range('a'..='z')).collect();
    let users = (0..n)
        .map(|_| User {
            name: (0..20).map(|_| rng.gen_range('a'..='z')).collect(),
            age: rng.gen_range(21..100),
            is_disabled: rng.gen_ratio(1, 4),
        })
        .collect();
    Users { title, users }
}

pub fn plain() -> Users {
    Users {
        title: "My awesome webpage!".to_owned(),
        users: vec![
            User {
                name: "Nancy Wheeler".to_owned(),
                age: 17,
                is_disabled: false,
            },
            User {
                name: "Steve Harrington".to_owned(),
                age: 18,
                is_disabled: false,
            },
            User {
                name: "Billy Hargrove".to_owned(),
                age: 19,
                is_disabled: true,
            },
        ],
    }
}

#[derive(serde::Serialize)]
pub struct Recurse {
    recurse: Option<Box<Recurse>>,
}

pub fn recurse(depth: usize) -> Recurse {
    if depth == 0 {
        Recurse { recurse: None }
    } else {
        Recurse {
            recurse: Some(Box::new(recurse(depth - 1))),
        }
    }
}
