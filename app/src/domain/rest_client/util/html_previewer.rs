use url::Url;

use crate::common::ui_utils::{create_cookie, get_browser_host_info, remove_cookie};

pub static FETCH_WRAPPER_JS: &[u8] = include_bytes!("fetchWrapper.js");

pub fn add_preview_scripts(html: &mut String) {
    let head_start_indexes = html.match_indices("<head>").map(|p| p.0).collect::<Vec<usize>>();
    let head_end_indexes = html.match_indices("</head>").map(|p| p.0).collect::<Vec<usize>>();
    if head_start_indexes.len() == 1 && head_end_indexes.len() == 1 {
        let script_text = String::from_utf8_lossy(FETCH_WRAPPER_JS).to_string();

        html.insert_str(
            head_start_indexes[0] + 6,
            &format!("<script lang=\"javascript\">{}</script>", script_text),
        );
    }
}

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

pub fn replace_absolute_links(html: &mut String, base_url: &str) {
    replace_absolute_links_by_attr_part(html, "href=\"", "\"", base_url);
    replace_absolute_links_by_attr_part(html, "href='", "'", base_url);

    replace_absolute_links_by_attr_part(html, "src=\"", "\"", base_url);
    replace_absolute_links_by_attr_part(html, "src='", "'", base_url);

    replace_absolute_links_by_attr_part(html, "background:url(", ")", base_url);
    replace_absolute_links_by_attr_part(html, "background:url('", "'", base_url);
    replace_absolute_links_by_attr_part(html, "background:url(\"", "\"", base_url);
    replace_absolute_links_by_attr_part(html, "background: url(", ")", base_url);
    replace_absolute_links_by_attr_part(html, "background: url('", "'", base_url);
    replace_absolute_links_by_attr_part(html, "background: url(\"", "\"", base_url);

    replace_absolute_links_by_attr_part(html, "background-image:url(", ")", base_url);
    replace_absolute_links_by_attr_part(html, "background-image:url('", "'", base_url);
    replace_absolute_links_by_attr_part(html, "background-image:url(\"", "\"", base_url);
    replace_absolute_links_by_attr_part(html, "background-image: url(", ")", base_url);
    replace_absolute_links_by_attr_part(html, "background-image: url('", "'", base_url);
    replace_absolute_links_by_attr_part(html, "background-image: url(\"", "\"", base_url);
}

pub fn init_html_previewer(proxy_allow: bool, base_url: &str) -> Result<(), String> {
    if proxy_allow {
        create_cookie("rc_base_url", &build_base_url(base_url), None)?;
    }

    Ok(())
}

pub fn clear_html_previewer() {
    remove_cookie("rc_base_url", "/");
}

fn replace_absolute_links_by_attr_part(
    html: &mut String,
    start_attr_part: &str,
    end_attr_part: &str,
    base_url: &str,
) {
    let indexes = html.match_indices(&start_attr_part).map(|p| p.0).collect::<Vec<usize>>();
    let mut offset: i32 = 0;
    for i in indexes {
        let start_index = (i as i32 + start_attr_part.len() as i32 + offset) as usize;
        if let Some(end_index) = find_from_byte_index(html, start_index, end_attr_part)
            && let Some(href) = html.get(start_index..end_index)
            && let Some(url) = convert_url(href, base_url)
        {
            html.replace_range(start_index..end_index, &url);
            offset += url.len() as i32 - (end_index - start_index) as i32
        }
    }
}

fn convert_url(url: &str, base_url: &str) -> Option<String> {
    if is_absolute_url(url) {
        if let Some(converted_url) = convert_absolute_url(url) {
            return Some(converted_url);
        }
    } else if is_special_url(url) {
        return None;
    }

    if !url.starts_with("/")
        && let Ok(base_url) = Url::parse(base_url)
        && let Ok(url) = base_url.join(url)
    {
        return Some(format!(
            "{}{}",
            url.path(),
            match url.query() {
                Some(query) => format!("?{}", query),
                None => "".to_owned(),
            }
        ));
    }

    None
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
    if is_special_url(src_url) {
        return Some(src_url.to_owned());
    }

    if let Ok(mut url) = Url::parse(src_url)
        && let Ok(host_info) = get_browser_host_info()
    {
        url.set_scheme(&host_info.0)
            .unwrap_or_else(|_| panic!("Cant set url scheme {}", host_info.0));
        url.set_host(Some(&host_info.1))
            .unwrap_or_else(|_| panic!("Cant set url host {} ", host_info.1));
        url.set_port(host_info.2).unwrap_or_else(|_| panic!("Cant set url port {:?}", host_info.2));
        url.query_pairs_mut().append_pair("rc_src_url", src_url);
        return Some(url.to_string());
    }

    Some(src_url.to_owned())
}

fn is_special_url(url: &str) -> bool {
    url.starts_with("'")
        || url.starts_with("\"")
        || url.starts_with("data:")
        || url.starts_with("javascript:")
        || url.starts_with("mailto:")
        || url.starts_with("tel:")
        || url.starts_with('#')
}
