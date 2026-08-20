use url::Url;

pub fn make_absolute_links(html: &mut String, base_url: &str) {
    make_absolute_links_by_attr(html, base_url, "href");
    make_absolute_links_by_attr(html, base_url, "src");


    make_absolute_links_by_attr_part(html, base_url, "background:url(", ")");
    make_absolute_links_by_attr_part(html, base_url, "background:url('", "'");
    make_absolute_links_by_attr_part(html, base_url, "background:url(\"", "\"");
    make_absolute_links_by_attr_part(html, base_url, "background: url(", ")");
    make_absolute_links_by_attr_part(html, base_url, "background: url('", "'");
    make_absolute_links_by_attr_part(html, base_url, "background: url(\"", "\"");

    make_absolute_links_by_attr_part(html, base_url, "background-image:url(", ")");
    make_absolute_links_by_attr_part(html, base_url, "background-image:url('", "'");
    make_absolute_links_by_attr_part(html, base_url, "background-image:url(\"", "\"");
    make_absolute_links_by_attr_part(html, base_url, "background-image: url(", ")");
    make_absolute_links_by_attr_part(html, base_url, "background-image: url('", "'");
    make_absolute_links_by_attr_part(html, base_url, "background-image: url(\"", "\"");
}

fn make_absolute_links_by_attr(html: &mut String, base_url: &str, attr_name: &str) {
    make_absolute_links_by_attr_part(html, base_url, &format!("{}=\"", attr_name), "\"");
    make_absolute_links_by_attr_part(html, base_url, &format!("{}='", attr_name), "'");

    add_head_base(html, base_url);
}

fn add_head_base(html: &mut String, base_url: &str) {
    let head_start_indexes = html.match_indices("<head>").map(|p| p.0).collect::<Vec<usize>>();
    let head_end_indexes = html.match_indices("</head>").map(|p| p.0).collect::<Vec<usize>>();
    if head_start_indexes.len() == 1 && head_end_indexes.len() == 1 {
        let head_inner = &html[head_start_indexes[0]..head_end_indexes[0]];
        let mut base_index = head_inner.match_indices("<base ");
        if base_index.next().is_none() {
            // insert <base>
            if let Some(base_url) = Url::parse(base_url).ok() && let Some(host) = base_url.host_str() {
                let base_url = format!("{}://{}", base_url.scheme(), host);
                html.insert_str(head_start_indexes[0] + 6, &format!("<base href=\"{}\" target=\"_blank\">", base_url));
            }
        }
    }
}

fn make_absolute_links_by_attr_part(
    html: &mut String,
    base_url: &str,
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
            if !is_absolute_url(href)
                && let Some(absolute) = resolve_absolute(href, base_url)
            {
                html.replace_range(start_index..end_index, &absolute);
                offset += absolute.len() - (end_index - start_index)
            } else {

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

fn resolve_absolute(relative: &str, base: &str) -> Option<String> {
    let relative = relative.trim();
    let base = base.trim();

    if relative.is_empty() {
        return None;
    }

    // Already absolute
    if is_absolute_url(relative) {
        if relative.starts_with("//") {
            return Some(format!("https:{relative}"));
        }
        return Some(relative.to_string());
    }

    // Special URLs
    if relative.starts_with("data:")
        || relative.starts_with("javascript:")
        || relative.starts_with("mailto:")
        || relative.starts_with("tel:")
        || relative.starts_with('#')
    {
        return Some(relative.to_string());
    }

    let base_url = Url::parse(base).ok()?;
    let resolved = base_url.join(relative).ok()?;

    Some(resolved.to_string())
}
