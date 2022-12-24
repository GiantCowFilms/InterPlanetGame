use rand::{ distributions::Alphanumeric, Rng };

pub fn generate_rejoin_code () -> String {
    let s: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(7)
        .map(char::from)
        .collect();
    return s;
}