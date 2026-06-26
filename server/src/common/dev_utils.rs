use std::collections::HashMap;

pub fn parse_query_params<'a>(query_str: &'a str) -> HashMap<&'a str, &'a str> {
    query_str
        .split('&')
        .filter(|pair| !pair.is_empty())
        .filter_map(|pair| {
            let mut parts = pair.splitn(2, '=');
            let key = parts.next()?;
            let value = parts.next().unwrap_or("");
            Some((key, value))
        })
        .collect()
}
