use chrono::NaiveDate;
use data_model::{Event, Programme};
use reqwest::Url;
use reqwest::header::{ACCEPT, ACCEPT_LANGUAGE, CACHE_CONTROL, HeaderMap, HeaderValue};

use crate::error::Error;
use crate::parser::{parse_programmes, parse_schedule_xml};

const DEFAULT_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) \
    AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36";

const SCHEDULE_PATH: &str = "setup/jsp/SchemaXML.jsp";
const PROGRAMMES_PATH: &str = "ajax/ajax_sokResurser.jsp";

const REQUEST_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

#[derive(Clone)]
pub struct Client {
    http: reqwest::Client,
}

impl Client {
    /// Build a client with the browser-like headers KronoX expects.
    ///
    /// # Errors
    /// Returns an error if the underlying TLS/HTTP client cannot be built.
    pub fn new() -> Result<Self, Error> {
        let mut headers = HeaderMap::new();
        headers.insert(
            ACCEPT,
            HeaderValue::from_static(
                "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
            ),
        );
        headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US"));
        headers.insert(CACHE_CONTROL, HeaderValue::from_static("no-cache"));

        let http = reqwest::ClientBuilder::new()
            .user_agent(DEFAULT_USER_AGENT)
            .default_headers(headers)
            .redirect(reqwest::redirect::Policy::none())
            .timeout(REQUEST_TIMEOUT)
            .build()?;
        Ok(Client { http })
    }

    /// Fetch and parse schedule events for the given schedule IDs.
    ///
    /// `start_date` of `None` means "today" (`startDatum=idag`).
    ///
    /// # Errors
    /// Returns an error if the request fails, KronoX returns a non-success
    /// status, or the response XML cannot be parsed.
    pub async fn fetch_events(
        &self,
        base_url: &str,
        school_code: &str,
        schedule_ids: &[String],
        start_date: Option<NaiveDate>,
    ) -> Result<Vec<Event>, Error> {
        let start_datum = match start_date {
            Some(date) => date.format("%Y-%m-%d").to_string(),
            None => "idag".to_owned(),
        };
        let resurser = schedule_ids.join(",");
        let params = [
            ("startDatum", start_datum.as_str()),
            ("intervallTyp", "a"),
            ("intervallAntal", "1"),
            ("sprak", "EN"),
            ("sokMedAND", "false"),
            ("forklaringar", "true"),
            ("resurser", resurser.as_str()),
        ];

        let xml = self.get(base_url, SCHEDULE_PATH, &params).await?;
        parse_schedule_xml(school_code, schedule_ids, &xml)
    }

    /// Free-text programme search.
    ///
    /// # Errors
    /// Returns an error if the request fails or KronoX returns a non-success status.
    pub async fn search_programmes(
        &self,
        base_url: &str,
        query: &str,
    ) -> Result<Vec<Programme>, Error> {
        let params = [
            ("sokord", query),
            ("startDatum", "idag"),
            ("slutDatum", ""),
            ("intervallTyp", "m"),
            ("intervallAntal", "6"),
        ];

        let html = self.get(base_url, PROGRAMMES_PATH, &params).await?;
        Ok(parse_programmes(&html))
    }

    async fn get(
        &self,
        base_url: &str,
        path: &str,
        params: &[(&str, &str)],
    ) -> Result<String, Error> {
        let url = Url::parse(&format!("{}/{}", base_url.trim_end_matches('/'), path))?;
        let response = self.http.get(url).query(params).send().await?;
        let status = response.status();
        if !status.is_success() {
            return Err(Error::Upstream {
                status: status.as_u16(),
            });
        }
        Ok(response.text().await?)
    }
}
