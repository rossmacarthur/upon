use rand::Rng;

#[derive(serde::Serialize)]
pub struct Context {
    pub title: String,
    pub users: Vec<User>,
}

#[derive(serde::Serialize)]
pub struct User {
    pub name: String,
    pub age: u32,
    pub is_disabled: bool,
}

pub fn random(n: usize) -> Context {
    let mut rng = rand::thread_rng();
    let title = (0..20).map(|_| rng.gen_range('a'..='z')).collect();
    let users = (0..n)
        .map(|_| User {
            name: (0..20).map(|_| rng.gen_range('a'..='z')).collect(),
            age: rng.gen_range(21..100),
            is_disabled: rng.gen_ratio(1, 4),
        })
        .collect();
    Context { title, users }
}
