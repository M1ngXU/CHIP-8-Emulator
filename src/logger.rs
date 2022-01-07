use std::fmt::Debug;

pub trait LogError {
    fn elog(self, msg: &str);
}
impl<T, E: Debug> LogError for Result<T, E> {
    fn elog(self, msg: &str) {
        if let Err(e) = self {
            eprintln!("ERROR while {}: {:?}", msg, e);
        }
    }
}

pub trait LogWarning {
    fn wlog(self);
}
impl LogWarning for &str {
    fn wlog(self) {
        println!("WARNING: {}", self);
    }
}

pub trait LogInfo {
    fn log(self);
}
impl LogInfo for &str {
    fn log(self) {
        println!("INFO: {}", self);
    }
}