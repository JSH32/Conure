pub use dotenv::dotenv;

use std::{
    env,
    fmt::Debug,
    net::{SocketAddr, ToSocketAddrs},
    str::FromStr,
};

/// Server config
pub struct Config {
    pub address: SocketAddr,
}

impl Config {
    pub fn load() -> Self {
        dotenv().unwrap();

        Self {
            address: {
                get_env::<String>("ADDRESS")
                    .to_socket_addrs()
                    .unwrap()
                    .next()
                    .expect("could not parse address")
            },
        }
    }
}

/// Get env as [`T`] or `default`.
pub fn get_env_or<T>(var: &str, default: T) -> T
where
    T: FromStr,
    <T as FromStr>::Err: Debug,
{
    match env::var(var) {
        Ok(v) => v.parse::<T>().expect(&format!(
            "Unable to parse {} as {}",
            var,
            std::any::type_name::<T>()
        )),
        Err(_) => default,
    }
}

/// Get env as [`T`].
pub fn get_env<T>(var: &str) -> T
where
    T: FromStr,
    <T as FromStr>::Err: Debug,
{
    env::var(var)
        .expect(&format!("Missing environment variable {}", var))
        .parse::<T>()
        .expect(&format!(
            "Unable to parse {} as {}",
            var,
            std::any::type_name::<T>()
        ))
}
