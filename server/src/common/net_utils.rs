use crate::common::app_error::AppError;

pub fn get_local_addrs() -> Result<Vec<(String, String)>, AppError> {
    use getifs::local_addrs;

    let mut result = Vec::new();
    for addr in local_addrs()? {
        result.push((
            addr.addr().to_string(),
            addr.name()?.to_string(),
        ));
    }

    Ok(result)
}
