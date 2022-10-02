lazy_static::lazy_static! {
    pub static ref API_URL: String = std::env::var("API_URL").unwrap_or_else(|_| String::from("localhost:5900"));
}
