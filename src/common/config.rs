pub fn env_var<T>(name: &str) -> T
where
    T: std::str::FromStr,
    T::Err: std::fmt::Debug,
{
    std::env::var(name)
        .unwrap_or_else(|_| panic!("{} env variable load failed", name))
        .parse::<T>()
        .unwrap_or_else(|_| panic!("{} env variable parse failed", name))
}

pub fn env_var_default<T>(name: &str, default_value: T) -> T
where
    T: std::str::FromStr,
    T::Err: std::fmt::Debug,
{
    match std::env::var(name) {
        Ok(var) => var.parse::<T>().unwrap_or_else(|_| {
            tracing::error!("{} env variable parse failed", name);
            default_value
        }),
        Err(_) => {
            tracing::error!("{} env variable load failed", name);
            default_value
        }
    }
}
