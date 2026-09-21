use std::{collections::HashMap, time::Instant};

pub struct Db {
    pub data: HashMap<String, Entry>,
}

pub struct Entry {
    pub value: String,
    pub expires_at: Option<Instant>,
}
