use url::Url;

use crate::common::ui_utils::get_browser_host_info;

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

pub fn build_base_url(url: &str) -> String {
    if let Ok(mut base_url) = Url::parse(url) {
        base_url.set_query(None);
        base_url.to_string()
    } else {
        url.to_owned()
    }
}

pub fn replace_absolute_links(html: &mut String) {
    replace_absolute_links_by_attr(html, "href");
    replace_absolute_links_by_attr(html, "src");


    replace_absolute_links_by_attr_part(html, "background:url(", ")");
    replace_absolute_links_by_attr_part(html, "background:url('", "'");
    replace_absolute_links_by_attr_part(html, "background:url(\"", "\"");
    replace_absolute_links_by_attr_part(html, "background: url(", ")");
    replace_absolute_links_by_attr_part(html, "background: url('", "'");
    replace_absolute_links_by_attr_part(html, "background: url(\"", "\"");

    replace_absolute_links_by_attr_part(html, "background-image:url(", ")");
    replace_absolute_links_by_attr_part(html, "background-image:url('", "'");
    replace_absolute_links_by_attr_part(html, "background-image:url(\"", "\"");
    replace_absolute_links_by_attr_part(html, "background-image: url(", ")");
    replace_absolute_links_by_attr_part(html, "background-image: url('", "'");
    replace_absolute_links_by_attr_part(html, "background-image: url(\"", "\"");
}

fn replace_absolute_links_by_attr(html: &mut String, attr_name: &str) {
    replace_absolute_links_by_attr_part(html, &format!("{}=\"", attr_name), "\"");
    replace_absolute_links_by_attr_part(html, &format!("{}='", attr_name), "'");
}

fn replace_absolute_links_by_attr_part(
    html: &mut String,
    start_attr_part: &str,
    end_attr_part: &str,
) {
    let indexes = html.match_indices(&start_attr_part).map(|p| p.0).collect::<Vec<usize>>();
    let mut offset = 0;
    for i in indexes {
        let start_index = i + start_attr_part.len() + offset;
        if let Some(end_index) = find_from_byte_index(html, start_index, end_attr_part)
            && let Some(href) = html.get(start_index..end_index)
        {
            if is_absolute_url(href)
                && let Some(absolute) = convert_absolute_url(href)
            {
                html.replace_range(start_index..end_index, &absolute);
                offset += absolute.len() - (end_index - start_index)
            }
        }
    }
}

fn find_from_byte_index(haystack: &str, start_at: usize, needle: &str) -> Option<usize> {
    haystack
        .get(start_at..)
        .and_then(|sub_slice| sub_slice.find(needle))
        .map(|relative_index| start_at + relative_index)
}

fn is_absolute_url(url_str: &str) -> bool {
    let url_str = url_str.trim();
    url_str.starts_with("http://") || url_str.starts_with("https://") || url_str.starts_with("//")
}

fn convert_absolute_url(src_url: &str) -> Option<String> {
    let src_url = src_url.trim();

    if src_url.is_empty() {
        return None;
    }

    // None absolute
    if !is_absolute_url(src_url) {
        return Some(src_url.to_owned());
    }

    // Special URLs
    if src_url.starts_with("data:")
        || src_url.starts_with("javascript:")
        || src_url.starts_with("mailto:")
        || src_url.starts_with("tel:")
        || src_url.starts_with('#')
    {
        return Some(src_url.to_owned());
    }

    if let Ok(mut url) = Url::parse(src_url) {
        let host_info = get_browser_host_info();

        url.set_scheme(&host_info.0).unwrap();
        url.set_host(Some(&host_info.1)).unwrap();
        url.set_port(Some(host_info.2)).unwrap();
        url.query_pairs_mut().append_pair("rc_base_url", src_url);
        return Some(url.to_string());
    }

    Some(src_url.to_owned())
}

