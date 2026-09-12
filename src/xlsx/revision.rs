use quick_xml::events::Event;
use quick_xml::Reader;

use super::{attr, local, text_of};
use crate::{Cell, Revision, RevisionChange, RevisionLog, RevisionRowColumnAction};

#[derive(PartialEq)]
enum ParserState {
    /// Value <v>
    V,
    /// Formula <f>
    F,
}

#[derive(Debug, Clone)]
enum RevisionChangeBuilder {
    CellChange {
        sid: usize,
        changes: Vec<RevisionChangeBuilder>,
    },
    NewCell {
        address: String,
        value: Option<Cell>,
    },
    RowColumn {
        sid: usize,
        /// Action taken like Insert or Delete
        action: RevisionRowColumnAction,
        /// Address of inserted or deleted item
        address: String,
        /// Changes
        changes: Vec<RevisionChangeBuilder>,
    },
}

impl RevisionChangeBuilder {
    pub fn build(self) -> Option<RevisionChange> {
        match self {
            RevisionChangeBuilder::CellChange { sid, changes } => {
                Some(RevisionChange::CellChange {
                    sid,
                    changes: changes.into_iter().filter_map(|c| c.build()).collect(),
                })
            }
            RevisionChangeBuilder::NewCell { address, value } => Some(RevisionChange::NewCell {
                address,
                value: value?,
            }),
            RevisionChangeBuilder::RowColumn {
                action,
                address,
                sid,
                changes,
            } => Some(RevisionChange::RowColumn {
                action,
                address,
                sid,
                changes: changes.into_iter().filter_map(|c| c.build()).collect(),
            }),
        }
    }
}

#[derive(Debug, Clone)]
enum RevisionBuilder {
    /// Insert or delete row or column
    RowColumn {
        rid: usize,
        sid: usize,
        /// Action taken like Insert or Delete
        action: RevisionRowColumnAction,
        /// Address of inserted or deleted item
        address: String,
        /// Changes
        changes: Vec<RevisionChangeBuilder>,
    },
    /// Change cell
    CellChange {
        rid: usize,
        sid: usize,
        /// Changes
        changes: Vec<RevisionChangeBuilder>,
    },
}

impl RevisionBuilder {
    pub fn build(self) -> Option<Revision> {
        match self {
            RevisionBuilder::RowColumn {
                action,
                address,
                rid,
                sid,
                changes,
            } => Some(Revision::RowColumn {
                rid,
                sid,
                action,
                address,
                changes: changes.into_iter().filter_map(|c| c.build()).collect(),
            }),

            RevisionBuilder::CellChange { rid, sid, changes } => Some(Revision::CellChange {
                rid,
                sid,
                changes: changes.into_iter().filter_map(|c| c.build()).collect(),
            }),
        }
    }

    pub fn push_change(&mut self, change: RevisionChangeBuilder) {
        match self {
            RevisionBuilder::RowColumn { changes, .. }
            | RevisionBuilder::CellChange { changes, .. } => changes.push(change),
        }
    }
}

#[derive(Debug)]
pub(super) struct RevisionRef {
    pub guid: String,
    pub date_time: String,
    pub user_name: String,
    pub rid: String,
    pub min_rid: Option<usize>,
    pub max_rid: Option<usize>,
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
                        min_rid: attr(&e, b"minRId").and_then(|value| value.parse::<usize>().ok()),
                        max_rid: attr(&e, b"maxRId").and_then(|value| value.parse::<usize>().ok()),
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
                        min_rid: attr(&e, b"minRId").and_then(|value| value.parse::<usize>().ok()),
                        max_rid: attr(&e, b"maxRId").and_then(|value| value.parse::<usize>().ok()),
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

pub(super) fn parse_revision(xml: &str, revision_ref: &RevisionRef) -> RevisionLog {
    let mut r = Reader::from_str(xml);
    let mut revisions: Vec<RevisionBuilder> = Vec::new();
    let mut parser_state: Option<ParserState> = None;

    let mut current_revision: Option<RevisionBuilder> = None;
    let mut changes_stack: Vec<RevisionChangeBuilder> = Vec::new();

    loop {
        match r.read_event() {
            Ok(Event::Start(e)) => match local(e.name().as_ref()) {
                b"rcc" => {
                    let sid = attr(&e, b"sId")
                        .and_then(|value| value.parse::<usize>().ok())
                        .unwrap_or_default();
                    if current_revision.is_some() {
                        let current_change = RevisionChangeBuilder::CellChange {
                            sid,
                            changes: Vec::new(),
                        };
                        changes_stack.push(current_change);
                    } else {
                        current_revision = Some(RevisionBuilder::CellChange {
                            rid: attr(&e, b"rId")
                                .and_then(|value| value.parse::<usize>().ok())
                                .unwrap_or_default(),
                            sid,
                            changes: Vec::new(),
                        })
                    }
                }

                b"nc" => {
                    let change = RevisionChangeBuilder::NewCell {
                        value: None,
                        address: attr(&e, b"r").unwrap_or_default(),
                    };
                    changes_stack.push(change);
                }

                b"v" | b"t" => {
                    parser_state = Some(ParserState::V);
                }
                b"f" => {
                    parser_state = Some(ParserState::F);
                }

                _ => {}
            },

            #[allow(clippy::single_match)]
            Ok(Event::Empty(e)) => match local(e.name().as_ref()) {
                b"rrc" => {
                    let sid = attr(&e, b"sId")
                        .and_then(|value| value.parse::<usize>().ok())
                        .unwrap_or_default();
                    let action = match attr(&e, b"action").unwrap_or_default().as_str() {
                        "insertRow" => RevisionRowColumnAction::InsertRow,
                        "deleteRow" => RevisionRowColumnAction::DeleteRow,
                        "insertCol" => RevisionRowColumnAction::InsertCol,
                        "deleteCol" => RevisionRowColumnAction::DeleteCol,
                        _ => RevisionRowColumnAction::InsertRow,
                    };
                    if let Some(current_revision) = current_revision.as_mut() {
                        current_revision.push_change(RevisionChangeBuilder::RowColumn {
                            address: attr(&e, b"ref").unwrap_or_default(),
                            sid,
                            action,
                            changes: Vec::new(),
                        })
                    } else {
                        revisions.push(RevisionBuilder::RowColumn {
                            address: attr(&e, b"ref").unwrap_or_default(),
                            rid: attr(&e, b"rId")
                                .and_then(|value| value.parse::<usize>().ok())
                                .unwrap_or_default(),
                            sid,
                            action,
                            changes: Vec::new(),
                        });
                    }
                }

                _ => {}
            },

            Ok(Event::Text(e)) => {
                if let Some(state) = parser_state.as_ref() {
                    if let Some(current_change) = &mut changes_stack.last_mut() {
                        match state {
                            ParserState::V => {
                                if let RevisionChangeBuilder::NewCell { value, .. } = current_change
                                {
                                    *value = Some(Cell::Text(text_of(&e)));
                                }
                            }
                            ParserState::F => {
                                if let RevisionChangeBuilder::NewCell { value, .. } = current_change
                                {
                                    *value = Some(Cell::Formula {
                                        formula: text_of(&e),
                                        cached: Box::new(Cell::Text("".to_string())),
                                    });
                                }
                            }
                        }
                    }
                }
            }

            Ok(Event::End(e)) => match local(e.name().as_ref()) {
                b"v" | b"f" | b"t" => parser_state = None,

                b"rcc" => {
                    if let Some(last_change) = changes_stack
                        .pop_if(|c| matches!(c, RevisionChangeBuilder::RowColumn { .. }))
                    {
                        current_revision.as_mut().unwrap().push_change(last_change);
                    } else if let Some(current_revision) = current_revision.take() {
                        revisions.push(current_revision);
                    }
                }

                b"nc" => {
                    if let Some(last_change) =
                        changes_stack.pop_if(|c| matches!(c, RevisionChangeBuilder::NewCell { .. }))
                    {
                        current_revision.as_mut().unwrap().push_change(last_change);
                    }
                }

                _ => {}
            },

            Ok(Event::Eof) | Err(_) => break,

            _ => {}
        }
    }

    let revisions = revisions.into_iter().filter_map(|c| c.build()).collect();

    RevisionLog {
        guid: revision_ref.guid.clone(),
        date_time: revision_ref.date_time.clone(),
        min_rid: revision_ref.min_rid,
        max_rid: revision_ref.max_rid,
        revision_log_id: revision_ref.rid.clone(),
        user_name: revision_ref.user_name.clone(),
        revisions,
    }
}
