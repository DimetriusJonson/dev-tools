#[cfg(feature = "ssr")]
pub mod ssr {
    use crate::common::app_error::AppError;

    pub fn get_local_addrs() -> Result<Vec<(String, String)>, AppError> {
        use getifs::local_addrs;

        let mut result = Vec::new();
        for addr in local_addrs().map_err(AppError::system_error)? {
            result.push((addr.addr().to_string(), addr.name().unwrap().to_string()));
        }

        Ok(result)
    }
}
