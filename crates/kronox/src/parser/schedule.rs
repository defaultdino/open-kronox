use std::collections::HashMap;
use std::sync::LazyLock;

use chrono::{DateTime, NaiveDateTime, Utc};
use data_model::{Event, Location, Teacher};
use regex::Regex;

use super::xml::{EventPost, ExplanationRow, KronoxScheduleXml};
use crate::error::Error;

const TIME_FORMAT: &str = "%Y%m%dT%H%M%SZ";

const DEFAULT_COLOR: &str = "#4A90E2";
const UNKNOWN: &str = "N/A";

static HTML_TAG_RE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"<.*?>").expect("valid regex"));

/// Parse the schedule document, dropping any entry whose timestamps don't parse.
///
/// # Errors
/// Returns [`Error::Xml`] if the document is not valid `KronoX` schedule XML.
pub fn parse_schedule_xml(
    school_code: &str,
    schedule_ids: &[String],
    xml: &str,
) -> Result<Vec<Event>, Error> {
    let document: KronoxScheduleXml = quick_xml::de::from_str(xml)?;

    let teachers = teacher_lookup(&document.explanation_texts.rows);
    let locations = location_lookup(&document.explanation_texts.rows);
    let courses = course_lookup(&document.explanation_texts.rows);

    let events = document
        .posts
        .iter()
        .filter_map(|post| {
            parse_event(
                school_code,
                schedule_ids,
                post,
                &teachers,
                &locations,
                &courses,
            )
            .ok()
        })
        .collect();
    Ok(events)
}

fn columns_of(rows: &[ExplanationRow], row_type: &str) -> Vec<HashMap<String, String>> {
    rows.iter()
        .filter(|row| row.r#type == row_type)
        .flat_map(|row| &row.rows)
        .map(|entry| {
            entry
                .columns
                .iter()
                .map(|column| (column.header.clone(), column.value.trim().to_owned()))
                .collect()
        })
        .collect()
}

fn course_lookup(rows: &[ExplanationRow]) -> HashMap<String, String> {
    columns_of(rows, "UTB_KURSINSTANS_GRUPPER")
        .into_iter()
        .filter_map(|mut columns| {
            let id = columns.remove("Id")?;
            let name = columns.remove("KursNamn_SV")?;
            (!id.is_empty() && !name.is_empty()).then_some((id, name))
        })
        .collect()
}

fn teacher_lookup(rows: &[ExplanationRow]) -> HashMap<String, Teacher> {
    columns_of(rows, "RESURSER_SIGNATURER")
        .into_iter()
        .filter_map(|mut columns| {
            let id = columns.remove("Id").filter(|id| !id.is_empty())?;
            let teacher = Teacher {
                id: id.clone(),
                first_name: columns.remove("ForNamn").unwrap_or_default(),
                last_name: columns.remove("EfterNamn").unwrap_or_default(),
            };
            Some((id, teacher))
        })
        .collect()
}

fn location_lookup(rows: &[ExplanationRow]) -> HashMap<String, Location> {
    columns_of(rows, "RESURSER_LOKALER")
        .into_iter()
        .filter_map(|mut columns| {
            let id = columns.remove("Id").filter(|id| !id.is_empty())?;
            let location = Location {
                id: id.clone(),
                name: columns.remove("Lokalnamn").unwrap_or_default(),
                building: columns.remove("Hus").unwrap_or_default(),
                floor: columns.remove("Vaning").unwrap_or_default(),
                max_seats: columns.remove("Antalplatser").unwrap_or_default(),
            };
            Some((id, location))
        })
        .collect()
}

fn parse_time(value: &str) -> Result<DateTime<Utc>, Error> {
    Ok(NaiveDateTime::parse_from_str(value, TIME_FORMAT)?.and_utc())
}

#[derive(Default)]
struct Resources {
    course_id: String,
    course_name: String,
    teachers: Vec<Teacher>,
    locations: Vec<Location>,
    schedule_ids: Vec<String>,
}

fn resolve_resources(
    post: &EventPost,
    teachers: &HashMap<String, Teacher>,
    locations: &HashMap<String, Location>,
    courses: &HashMap<String, String>,
) -> Resources {
    let mut resources = Resources::default();
    for node in &post.resource_row.resource_nodes {
        let id = clean_resource_id(&node.id);
        match node.type_id.as_str() {
            "UTB_KURSINSTANS_GRUPPER" => {
                if let Some(name) = courses.get(&id) {
                    resources.course_name = name.clone();
                }
                resources.course_id = id;
            }
            "RESURSER_SIGNATURER" => {
                if let Some(teacher) = teachers.get(&id) {
                    resources.teachers.push(teacher.clone());
                }
            }
            "RESURSER_LOKALER" => {
                if let Some(location) = locations.get(&id) {
                    resources.locations.push(location.clone());
                }
            }
            "UTB_PROGRAMINSTANS_KLASSER" => {
                if node.url_encoded_id.is_empty() {
                    resources.schedule_ids.push(id);
                } else {
                    resources.schedule_ids.push(node.url_encoded_id.clone());
                }
            }
            _ => {}
        }
    }
    resources
}

/// resource ids sometimes arrive wrapped in a CDATA section.. strip it!
fn clean_resource_id(raw: &str) -> String {
    raw.trim()
        .trim_start_matches("<![CDATA[")
        .trim_end_matches("]]>")
        .trim()
        .to_owned()
}

fn parse_event(
    school_code: &str,
    schedule_ids: &[String],
    post: &EventPost,
    teachers: &HashMap<String, Teacher>,
    locations: &HashMap<String, Location>,
    courses: &HashMap<String, String>,
) -> Result<Event, Error> {
    let from = parse_time(&post.booked_dates.start_date_time)?;
    let to = parse_time(&post.booked_dates.end_date_time)?;
    let last_modified = parse_time(&post.last_modified).unwrap_or_default();

    let title = HTML_TAG_RE.replace_all(&post.moment, "").trim().to_owned();
    let resources = resolve_resources(post, teachers, locations, courses);

    let schedule_id = match_schedule_id(schedule_ids, &resources.schedule_ids)
        .or_else(|| schedule_ids.first().cloned())
        .unwrap_or_default();

    Ok(Event {
        event_id: post.booking_id.clone(),
        schedule_id,
        title,
        course_id: non_empty_or_unknown(resources.course_id),
        course_name: non_empty_or_unknown(resources.course_name),
        teachers: resources.teachers,
        from,
        to,
        locations: resources.locations,
        last_modified,
        is_special: false,
        school_code: school_code.to_owned(),
        color_hex: DEFAULT_COLOR.to_owned(),
    })
}

fn non_empty_or_unknown(value: String) -> String {
    if value.is_empty() {
        UNKNOWN.to_owned()
    } else {
        value
    }
}

fn match_schedule_id(requested: &[String], from_xml: &[String]) -> Option<String> {
    from_xml.iter().find_map(|xml_id| {
        requested.iter().find_map(|requested_id| {
            let matches = match requested_id.split_once('.') {
                Some((_, suffix)) => suffix == xml_id,
                None => requested_id == xml_id,
            };
            matches.then(|| requested_id.clone())
        })
    })
}

#[cfg(test)]
mod tests {
    use super::parse_schedule_xml;

    const SCHEDULE_XML: &str = r#"<schema>
      <schemaPost>
        <moment>Lecture &lt;b&gt;X&lt;/b&gt;</moment>
        <bokningsId>BK1</bokningsId>
        <bokadeDatum startDatumTid_iCal="20260722T080000Z" slutDatumTid_iCal="20260722T100000Z"/>
        <senastAndradDatum_iCal>20260701T000000Z</senastAndradDatum_iCal>
        <resursTrad>
          <resursNod resursTypId="UTB_KURSINSTANS_GRUPPER"><resursId>C1</resursId><resursIdURLEncoded>C1</resursIdURLEncoded></resursNod>
          <resursNod resursTypId="RESURSER_SIGNATURER"><resursId>T1</resursId><resursIdURLEncoded>T1</resursIdURLEncoded></resursNod>
          <resursNod resursTypId="RESURSER_LOKALER"><resursId>L1</resursId><resursIdURLEncoded>L1</resursIdURLEncoded></resursNod>
        </resursTrad>
      </schemaPost>
      <forklaringstexter>
        <forklaringsrader typ="UTB_KURSINSTANS_GRUPPER">
          <rad><kolumn rubrik="Id">C1</kolumn><kolumn rubrik="KursNamn_SV">Programmering</kolumn></rad>
        </forklaringsrader>
        <forklaringsrader typ="RESURSER_SIGNATURER">
          <rad><kolumn rubrik="Id">T1</kolumn><kolumn rubrik="ForNamn">Anna</kolumn><kolumn rubrik="EfterNamn">Svensson</kolumn></rad>
        </forklaringsrader>
        <forklaringsrader typ="RESURSER_LOKALER">
          <rad><kolumn rubrik="Id">L1</kolumn><kolumn rubrik="Lokalnamn">Sal 101</kolumn><kolumn rubrik="Hus">A</kolumn><kolumn rubrik="Vaning">1</kolumn><kolumn rubrik="Antalplatser">30</kolumn></rad>
        </forklaringsrader>
      </forklaringstexter>
    </schema>"#;

    #[test]
    fn parses_event_joining_explanation_tables() {
        let requested = vec!["p.TEST".to_owned()];
        let events = parse_schedule_xml("hkr", &requested, SCHEDULE_XML).unwrap();

        assert_eq!(events.len(), 1);
        let event = &events[0];
        assert_eq!(event.event_id, "BK1");
        assert_eq!(event.title, "Lecture X");
        assert_eq!(event.course_id, "C1");
        assert_eq!(event.course_name, "Programmering");
        assert_eq!(event.from.to_rfc3339(), "2026-07-22T08:00:00+00:00");
        assert_eq!(event.teachers.len(), 1);
        assert_eq!(event.teachers[0].last_name, "Svensson");
        assert_eq!(event.locations.len(), 1);
        assert_eq!(event.locations[0].name, "Sal 101");
        assert_eq!(event.school_code, "hkr");
        assert_eq!(event.schedule_id, "p.TEST");
    }
}
