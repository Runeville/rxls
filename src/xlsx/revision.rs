use quick_xml::events::Event;
use quick_xml::Reader;

use super::{attr, local, text_of};
use crate::{Cell, Revision, RevisionChange, RevisionChangeEnum, RevisionRowColumnAction};

#[derive(PartialEq)]
enum ParserState {
    V,
}

#[derive(Debug, Clone)]
enum RevisionChangeEnumBuilder {
    RevisionRowColumn {
        action: RevisionRowColumnAction,
        address: String,
    },
    RevisionCellChange {
        value: Option<Cell>,
        address: String,
    },
    Formatting {
        start: usize,
        length: usize,
        address: String,
        action: String,
    },
}

impl RevisionChangeEnumBuilder {
    pub fn build(self) -> Option<RevisionChangeEnum> {
        match self {
            RevisionChangeEnumBuilder::RevisionRowColumn { action, address } => {
                Some(RevisionChangeEnum::RevisionRowColumn { action, address })
            }

            RevisionChangeEnumBuilder::RevisionCellChange { value, address } => {
                Some(RevisionChangeEnum::RevisionCellChange {
                    value: value.as_ref()?.clone(),
                    address: address.clone(),
                })
            }

            RevisionChangeEnumBuilder::Formatting {
                start,
                length,
                address,
                action,
            } => Some(RevisionChangeEnum::Formatting {
                start,
                length,
                address,
                action,
            }),
        }
    }
}

struct RevisionChangeBuilder {
    rid: usize,
    sid: usize,
    change: Option<RevisionChangeEnumBuilder>,
}

impl RevisionChangeBuilder {
    pub fn build(self) -> Option<RevisionChange> {
        Some(RevisionChange {
            rid: self.rid,
            sid: self.sid,
            change: self.change?.build()?,
        })
    }
}

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

pub(super) fn parse_revision(xml: &str, revision_ref: &RevisionRef) -> Revision {
    let mut r = Reader::from_str(xml);
    let mut changes: Vec<RevisionChangeBuilder> = Vec::new();
    let mut parser_state: Option<ParserState> = None;

    let mut current_change: Option<RevisionChangeBuilder> = None;

    loop {
        match r.read_event() {
            Ok(Event::Start(e)) => match local(e.name().as_ref()) {
                b"rcc" => {
                    current_change = Some(RevisionChangeBuilder {
                        rid: attr(&e, b"rid")
                            .and_then(|value| value.parse::<usize>().ok())
                            .unwrap_or_default(),
                        sid: attr(&e, b"sid")
                            .and_then(|value| value.parse::<usize>().ok())
                            .unwrap_or_default(),
                        change: None,
                    })
                }

                b"nc" => {
                    if let Some(current_change) = current_change.as_mut() {
                        current_change.change =
                            Some(RevisionChangeEnumBuilder::RevisionCellChange {
                                value: None,
                                address: attr(&e, b"r").unwrap_or_default(),
                            })
                    }
                }

                b"v" => {
                    parser_state = Some(ParserState::V);
                }

                _ => {}
            },

            Ok(Event::Empty(e)) => match local(e.name().as_ref()) {
                b"rrc" => changes.push(RevisionChangeBuilder {
                    rid: attr(&e, b"rid")
                        .and_then(|value| value.parse::<usize>().ok())
                        .unwrap_or_default(),
                    sid: attr(&e, b"sid")
                        .and_then(|value| value.parse::<usize>().ok())
                        .unwrap_or_default(),
                    change: Some(RevisionChangeEnumBuilder::RevisionRowColumn {
                        action: RevisionRowColumnAction::InsertRow,
                        address: attr(&e, b"ref").unwrap_or_default(),
                    }),
                }),

                _ => {}
            },

            Ok(Event::Text(e)) => {
                if let Some(state) = parser_state.as_ref() {
                    match state {
                        ParserState::V => {
                            if let Some(current_change) = current_change.as_mut() {
                                if let Some(RevisionChangeEnumBuilder::RevisionCellChange {
                                    value,
                                    ..
                                }) = current_change.change.as_mut()
                                {
                                    {
                                        *value = Some(Cell::Text(text_of(&e)));
                                    }
                                }
                            }
                        }
                    }
                }
            }

            Ok(Event::End(e)) => match local(e.name().as_ref()) {
                b"v" => parser_state = None,

                b"rcc" => {
                    if let Some(current_change) = current_change.take() {
                        changes.push(current_change);
                    }
                }

                _ => {}
            },

            Ok(Event::Eof) | Err(_) => break,

            _ => {}
        }
    }

    let changes = changes.into_iter().filter_map(|c| c.build()).collect();

    Revision {
        revision_id: revision_ref.rid.clone(),
        user_name: revision_ref.user_name.clone(),
        changes,
    }
}
