use quick_xml::events::Event;
use quick_xml::Reader;

use super::{attr, local};

#[derive(Debug)]
pub(super) struct RevisionRef {
    pub guid: String,
    pub date_time: String,
    pub user_name: String,
    pub rid: String,
    pub min_rid: Option<String>,
    pub max_rid: Option<String>,
    pub sheet_ids: Vec<u32>,
}

/// Revisions metadata parsed before individual revisions are opened.
#[derive(Debug)]
pub(super) struct ParsedRevisionHeaders {
    pub(super) revisions: Vec<RevisionRef>,
}

/// Parse workbook properties, ordered sheets, and defined names.
pub(super) fn parse_revision_headers(xml: &str) -> ParsedRevisionHeaders {
    let mut r = Reader::from_str(xml);
    let mut revisions = Vec::new();

    let mut current_header: Option<RevisionRef> = None;

    loop {
        match r.read_event() {
            Ok(Event::Start(e)) => match local(e.name().as_ref()) {
                b"header" => {
                    current_header = Some(RevisionRef {
                        guid: attr(&e, b"guid").unwrap_or_default(),
                        date_time: attr(&e, b"dateTime").unwrap_or_default(),
                        user_name: attr(&e, b"userName").unwrap_or_default(),
                        rid: attr(&e, b"id").unwrap_or_default(),
                        min_rid: attr(&e, b"minRId"),
                        max_rid: attr(&e, b"maxRId"),
                        sheet_ids: Vec::new(),
                    });
                }

                b"sheetId" => {
                    if let Some(header) = current_header.as_mut() {
                        if let Some(id) =
                            attr(&e, b"val").and_then(|value| value.parse::<u32>().ok())
                        {
                            header.sheet_ids.push(id);
                        }
                    }
                }

                _ => {}
            },

            Ok(Event::Empty(e)) => match local(e.name().as_ref()) {
                b"header" => {
                    revisions.push(RevisionRef {
                        guid: attr(&e, b"guid").unwrap_or_default(),
                        date_time: attr(&e, b"dateTime").unwrap_or_default(),
                        user_name: attr(&e, b"userName").unwrap_or_default(),
                        rid: attr(&e, b"id").unwrap_or_default(),
                        min_rid: attr(&e, b"minRId"),
                        max_rid: attr(&e, b"maxRId"),
                        sheet_ids: Vec::new(),
                    });
                }

                b"sheetId" => {
                    if let Some(header) = current_header.as_mut() {
                        if let Some(id) =
                            attr(&e, b"val").and_then(|value| value.parse::<u32>().ok())
                        {
                            header.sheet_ids.push(id);
                        }
                    }
                }

                _ => {}
            },

            Ok(Event::End(e)) => {
                if local(e.name().as_ref()) == b"header" {
                    if let Some(header) = current_header.take() {
                        revisions.push(header);
                    }
                }
            }

            Ok(Event::Eof) | Err(_) => break,

            _ => {}
        }
    }

    ParsedRevisionHeaders { revisions }
}
