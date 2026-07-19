use std::cmp;

use difference::{Changeset, Difference};

pub fn compare_text(text1: &str, text2: &str) -> (String, String) {
    let Changeset { diffs, .. } = Changeset::new(text2, text1, "\n");

    let mut result1 = Vec::new();
    let mut result2 = Vec::new();
    for i in 0..diffs.len() {
        //println!("diff={:?}", diffs[i]);
        //println!("**********************************\n");
        match diffs[i] {
            difference::Difference::Same(ref x) => {
                for text in x.lines() {
                    result1.push(format!(
                        "<tr>{}{}</tr>",
                        render_td_num(result1.len() + 1),
                        render_td_text(&normalize_str(text))
                    ));
                    result2.push(format!(
                        "<tr>{}{}</tr>",
                        render_td_num(result2.len() + 1),
                        render_td_text(&normalize_str(text))
                    ));
                }
            }
            difference::Difference::Add(ref x) => {
                if i > 0 {
                    match diffs[i - 1] {
                        Difference::Rem(ref y) => {
                            let x_lines: Vec<&str> = x.lines().collect();
                            let y_lines: Vec<&str> = y.lines().collect();
                            for i in 0..cmp::max(x_lines.len(), y_lines.len()) {
                                let text_x = x_lines.get(i).unwrap_or(&"");
                                let text_y = y_lines.get(i).unwrap_or(&"");

                                let mut text_row1 = "".to_owned();
                                let mut text_row2 = "".to_owned();
                                let Changeset { diffs, .. } = Changeset::new(text_y, text_x, "");
                                for c in diffs {
                                    match c {
                                        Difference::Same(ref z) => {
                                            text_row1.push_str(&normalize_str(z).to_string());
                                            text_row2.push_str(&normalize_str(z).to_string());
                                        }
                                        Difference::Add(ref z) => {
                                            text_row1.push_str(&wrap_str(
                                                "<span class=\"bg-cyan-700\">",
                                                normalize_str(z),
                                                "</span>",
                                            ));
                                        }
                                        Difference::Rem(ref z) => {
                                            text_row2.push_str(&wrap_str(
                                                "<span class=\"bg-cyan-700\">",
                                                normalize_str(z),
                                                "</span>",
                                            ));
                                        }
                                    }
                                }

                                for (i, text) in text_row1.lines().enumerate() {
                                    result1.push(format!(
                                        "<tr>{}{}</tr>",
                                        render_td_changed_num(result1.len() + 1),
                                        render_td_text(text)
                                    ));
                                    if i > 0 {
                                        result2.push(format!(
                                            "<tr>{}{}</tr>",
                                            render_td_changed_num(result2.len() + 1),
                                            render_td_empty()
                                        ));
                                    }
                                }

                                for (i, text) in text_row2.lines().enumerate() {
                                    result2.push(format!(
                                        "<tr>{}{}</tr>",
                                        render_td_changed_num(result2.len() + 1),
                                        render_td_text(text)
                                    ));
                                    if i > 0 {
                                        result1.push(format!(
                                            "<tr>{}{}</tr>",
                                            render_td_changed_num(result1.len() + 1),
                                            render_td_empty(),
                                        ));
                                    }
                                }
                            }
                        }
                        _ => {
                            for text in x.lines() {
                                result1.push(format!(
                                    "<tr>{}{}</tr>",
                                    render_td_changed_num(result1.len() + 1),
                                    render_td_text(&wrap_str(
                                        "<span class=\"bg-cyan-700\">",
                                        normalize_str(text),
                                        "</span>"
                                    ))
                                ));

                                result2.push(format!(
                                    "<tr>{}{}</tr>",
                                    render_td_changed_num(result2.len() + 1),
                                    render_td_empty()
                                ));
                            }
                        }
                    };
                } else {
                    for text in x.lines() {
                        result1.push(format!(
                            "<tr>{}{}</tr>",
                            render_td_changed_num(result1.len() + 1),
                            render_td_text(&normalize_str(text))
                        ));
                    }
                }
            }
            difference::Difference::Rem(ref x) => {
                let mut need = false;
                if i + 1 < diffs.len() {
                    match diffs[i + 1] {
                        Difference::Add(_) => (),
                        _ => need = true,
                    }
                } else {
                    need = true;
                }

                if need {
                    for text in x.lines() {
                        result2.push(format!(
                            "<tr>{}{}</tr>",
                            render_td_changed_num(result2.len() + 1),
                            render_td_text(&wrap_str(
                                "<span class=\"bg-cyan-700\">",
                                normalize_str(text),
                                "</span>"
                            ))
                        ));
                    }
                }
            }
        }
    }

    result1.insert(0, "<table class=\"table-auto w-full bg-mygray \">".to_owned());
    result1.push("</table>".to_owned());

    result2.insert(0, "<table class=\"table-auto w-full bg-mygray \">".to_owned());
    result2.push("</table>".to_owned());

    (result1.join("\n"), result2.join("\n"))
}

fn render_td_changed_num(num: usize) -> String {
    format!("<td class=\"w-10 bg-green-700 border-r pl-2\">{}</td>", num)
}

fn render_td_num(num: usize) -> String {
    format!("<td class=\"w-10 border-r pl-2\">{}</td>", num)
}

fn render_td_text(text: &str) -> String {
    format!("<td class=\"w-auto pl-2\"><pre>{}</pre></td>", text)
}

fn render_td_empty() -> String {
    "<td class=\"w-auto pl-2\"></td>".to_string()
}

fn wrap_str(pre: &str, content: String, post: &str) -> String {
    if content.lines().count() > 1 {
        let mut res_lines = Vec::new();
        for line in content.lines() {
            res_lines.push(format!("{}{}{}", pre, line, post));
        }
        res_lines.join("\n")
    } else {
        format!("{}{}{}", pre, content, post)
    }
}

fn normalize_str(src: &str) -> String {
    let r = html_escape::encode_text(src);
    let s = r.trim_end_matches(['\r', '\n']);
    s.to_string()
}
