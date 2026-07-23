use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct EventPost {
    pub moment: String,

    #[serde(rename = "bokningsId")]
    pub booking_id: String,

    #[serde(rename = "bokadeDatum")]
    pub booked_dates: BookedDates,

    #[serde(rename = "senastAndradDatum_iCal")]
    pub last_modified: String,

    #[serde(rename = "resursTrad")]
    pub resource_row: ResourceRow,
}

#[derive(Debug, Deserialize)]
pub struct BookedDates {
    #[serde(rename = "@startDatumTid_iCal")]
    pub start_date_time: String,
    #[serde(rename = "@slutDatumTid_iCal")]
    pub end_date_time: String,
}

#[derive(Debug, Deserialize)]
pub struct ResourceRow {
    #[serde(rename = "resursNod", default)]
    pub resource_nodes: Vec<ResourceNode>,
}

#[derive(Debug, Deserialize)]
pub struct ResourceNode {
    #[serde(rename = "@resursTypId")]
    pub type_id: String,
    #[serde(rename = "resursId")]
    pub id: String,
    #[serde(rename = "resursIdURLEncoded")]
    pub url_encoded_id: String,
}

#[derive(Debug, Deserialize)]
pub struct ExplanationRow {
    #[serde(rename = "@typ")]
    pub r#type: String,
    #[serde(rename = "rad", default)]
    pub rows: Vec<Row>,
}

#[derive(Debug, Deserialize)]
pub struct Row {
    #[serde(rename = "kolumn", default)]
    pub columns: Vec<Column>,
}

#[derive(Debug, Deserialize)]
pub struct Column {
    #[serde(rename = "@rubrik")]
    pub header: String,
    #[serde(rename = "$text", default)]
    pub value: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename = "schema")]
pub struct KronoxScheduleXml {
    #[serde(rename = "schemaPost", default)]
    pub posts: Vec<EventPost>,

    #[serde(rename = "forklaringstexter")]
    pub explanation_texts: ExplanationTexts,
}

#[derive(Debug, Deserialize)]
pub struct ExplanationTexts {
    #[serde(rename = "forklaringsrader", default)]
    pub rows: Vec<ExplanationRow>,
}
