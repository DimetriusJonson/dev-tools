use url::Url;

pub fn add_head_base_tag(html: &mut String, url: &str) {
    let base_url = build_base_url(url);
    let head_start_indexes = html.match_indices("<head>").map(|p| p.0).collect::<Vec<usize>>();
    let head_end_indexes = html.match_indices("</head>").map(|p| p.0).collect::<Vec<usize>>();
    if head_start_indexes.len() == 1 && head_end_indexes.len() == 1 {
        let head_inner = &html[head_start_indexes[0]..head_end_indexes[0]];
        let mut base_index = head_inner.match_indices("<base ");
        if base_index.next().is_none() {
            // insert <base>
            html.insert_str(
                head_start_indexes[0] + 6,
                &format!("<base href=\"{}\" target=\"_blank\">", base_url),
            );
        }
    }
}

pub fn add_base_url_script(html: &mut String, base_url: &str) {
    let base_url = build_base_url(base_url);

    let head_start_indexes = html.match_indices("<head>").map(|p| p.0).collect::<Vec<usize>>();
    let head_end_indexes = html.match_indices("</head>").map(|p| p.0).collect::<Vec<usize>>();
    if head_start_indexes.len() == 1 && head_end_indexes.len() == 1 {
        let head_inner = &html[head_start_indexes[0]..head_end_indexes[0]];
        let mut base_index = head_inner.match_indices("<base ");
        if base_index.next().is_none() {
            // insert <base>
            html.insert_str(
                head_start_indexes[0] + 6,
                &format!("<script>document.cookie = 'rc_base_url={}';</script>", base_url),
            );
        }
    }
}

fn build_base_url(url: &str) -> String {
    if let Ok(mut base_url) = Url::parse(url) {
        base_url.set_query(None);
        base_url.to_string()
    } else {
        url.to_owned()
    }
}
