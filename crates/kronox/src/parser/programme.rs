use std::collections::HashSet;
use std::sync::LazyLock;

use data_model::Programme;
use regex::Regex;
use scraper::{Html, Selector};

static RESURSER_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"resurser=(.*)$").expect("valid regex"));

/// Extract programmes from a KronoX programme-search results page.
///
/// Each result is an `<a target="_blank">` whose `href` carries the schedule id
/// in a `resurser=` query parameter and whose text is `"<title>, <subtitle>"`.
#[must_use]
pub fn parse_programmes(html: &str) -> Vec<Programme> {
    let selector = Selector::parse("a[target='_blank']").expect("valid selector");
    let document = Html::parse_document(html);

    document
        .select(&selector)
        .skip(2)
        .filter_map(parse_programme_link)
        .collect()
}

fn parse_programme_link(element: scraper::ElementRef<'_>) -> Option<Programme> {
    let href = element.value().attr("href")?;
    let id = RESURSER_RE.captures(href)?.get(1)?.as_str().to_owned();

    let text = element.text().collect::<String>();
    let (title, subtitle) = text.trim().split_once(',')?;

    Some(Programme {
        id,
        title: title.trim().to_owned(),
        subtitle: remove_duplicate_words(subtitle.trim()),
    })
}

fn remove_duplicate_words(text: &str) -> String {
    let mut seen = HashSet::new();
    text.split_whitespace()
        .filter(|word| seen.insert(*word))
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::parse_programmes;

    #[test]
    fn skips_first_two_navigation_links() {
        let html = r#"
          <a target="_blank" href="?resurser=skip1">A, x</a>
          <a target="_blank" href="?resurser=skip2">B, y</a>
          <a target="_blank" href="?resurser=p.REAL">Data, Datateknik Datateknik</a>
        "#;
        let programmes = parse_programmes(html);
        assert_eq!(programmes.len(), 1);
        assert_eq!(programmes[0].id, "p.REAL");
        assert_eq!(programmes[0].title, "Data");
        assert_eq!(programmes[0].subtitle, "Datateknik"); // duplicate words collapsed
    }
}
