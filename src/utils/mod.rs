pub mod time {
    use chrono;

    pub fn get_current_time() -> chrono::NaiveDateTime {
        chrono::Local::now().naive_utc()
    }
}
