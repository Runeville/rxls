use quick_xml::events::Event;
use quick_xml::Reader;

use super::{attr, local, text_of};
use crate::{
    Cell, Revision, RevisionChange, RevisionDifferentialFormat, RevisionFont, RevisionLog,
    RevisionNumberFormat, RevisionRowColumnAction, RevisionViewAction, SheetName, User,
};

#[derive(PartialEq)]
enum ParserState {
    /// Value <v>
    V,
    /// Text <t>
    T,
    /// Formula <f>
    F,
    /// Font <font>
    Font,
}

#[derive(Debug, Clone)]
enum RevisionChangeBuilder {
    CellChange {
        sheet_id: usize,
        changes: Vec<RevisionChangeBuilder>,
    },
    NewCell {
        address: String,
        value: Option<Cell>,
        is_bool: bool,
    },
    OldCell {
        address: String,
        value: Option<Cell>,
        is_bool: bool,
    },
    RowColumn {
        sheet_id: usize,
        /// Action taken like Insert or Delete
        action: RevisionRowColumnAction,
        /// Address of inserted or deleted item
        address: String,
        /// Changes
        changes: Vec<RevisionChangeBuilder>,
    },
    NewDifferentialFormat(RevisionDifferentialFormat),
    OldDifferentialFormat(RevisionDifferentialFormat),
}

impl RevisionChangeBuilder {
    pub fn build(self) -> Option<RevisionChange> {
        match self {
            RevisionChangeBuilder::CellChange { sheet_id, changes } => {
                Some(RevisionChange::CellChange {
                    sheet_id,
                    changes: changes.into_iter().filter_map(|c| c.build()).collect(),
                })
            }
            RevisionChangeBuilder::NewCell { address, value, .. } => {
                Some(RevisionChange::NewCell { address, value })
            }
            RevisionChangeBuilder::OldCell { address, value, .. } => {
                Some(RevisionChange::OldCell {
                    address,
                    value: value?,
                })
            }
            RevisionChangeBuilder::RowColumn {
                action,
                address,
                sheet_id,
                changes,
            } => Some(RevisionChange::RowColumn {
                action,
                address,
                sheet_id,
                changes: changes.into_iter().filter_map(|c| c.build()).collect(),
            }),
            RevisionChangeBuilder::NewDifferentialFormat(formatting) => {
                Some(RevisionChange::NewDifferentialFormat(formatting))
            }
            RevisionChangeBuilder::OldDifferentialFormat(formatting) => {
                Some(RevisionChange::OldDifferentialFormat(formatting))
            }
        }
    }
}

#[derive(Debug, Clone)]
enum RevisionBuilder {
    /// Insert or delete row or column
    RowColumn {
        revision_id: usize,
        sheet_id: usize,
        /// Action taken like Insert or Delete
        action: RevisionRowColumnAction,
        /// Address of inserted or deleted item
        address: String,
        /// Changes
        changes: Vec<RevisionChangeBuilder>,
        /// End of list
        eol: bool,
    },
    /// Change cell
    CellChange {
        revision_id: usize,
        sheet_id: usize,
        /// Changes
        changes: Vec<RevisionChangeBuilder>,
    },
    RevisionView {
        guid: String,
        action: RevisionViewAction,
    },
    InsertSheet {
        revision_id: usize,
        sheet_id: usize,
        name: SheetName,
        sheet_position: usize,
    },
    RenameSheet {
        revision_id: usize,
        sheet_id: usize,
        old_name: SheetName,
        new_name: SheetName,
    },
    Formatting {
        sheet_id: usize,
        start: Option<usize>,
        length: Option<usize>,
        address: String,
        formatting: Option<RevisionDifferentialFormat>,
    },
}

impl RevisionBuilder {
    pub fn build(self) -> Option<Revision> {
        match self {
            RevisionBuilder::RowColumn {
                revision_id,
                sheet_id,
                action,
                address,
                changes,
                eol,
            } => Some(Revision::RowColumn {
                revision_id,
                sheet_id,
                action,
                address,
                changes: changes.into_iter().filter_map(|c| c.build()).collect(),
                eol,
            }),

            RevisionBuilder::CellChange {
                revision_id,
                sheet_id,
                changes,
            } => Some(Revision::CellChange {
                revision_id,
                sheet_id,
                changes: changes.into_iter().filter_map(|c| c.build()).collect(),
            }),
            RevisionBuilder::RevisionView { guid, action } => {
                Some(Revision::RevisionView { guid, action })
            }
            RevisionBuilder::InsertSheet {
                revision_id,
                sheet_id,
                name,
                sheet_position,
            } => Some(Revision::InsertSheet {
                revision_id,
                sheet_id,
                name,
                sheet_position,
            }),
            RevisionBuilder::RenameSheet {
                revision_id,
                sheet_id,
                old_name,
                new_name,
            } => Some(Revision::RenameSheet {
                revision_id,
                sheet_id,
                old_name,
                new_name,
            }),
            RevisionBuilder::Formatting {
                sheet_id,
                start,
                length,
                address,
                formatting,
            } => Some(Revision::Formatting {
                sheet_id,
                start,
                length,
                address,
                formatting,
            }),
        }
    }

    pub fn push_change(&mut self, change: RevisionChangeBuilder) {
        match self {
            RevisionBuilder::RowColumn { changes, .. }
            | RevisionBuilder::CellChange { changes, .. } => changes.push(change),
            RevisionBuilder::RevisionView { .. }
            | RevisionBuilder::InsertSheet { .. }
            | RevisionBuilder::Formatting { .. } => {}
            RevisionBuilder::RenameSheet { .. } => {}
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
                        guid: attr(&e, b"guid")
                            .unwrap_or_default()
                            .trim_matches(['{', '}'])
                            .to_string(),
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
                        guid: attr(&e, b"guid")
                            .unwrap_or_default()
                            .trim_matches(['{', '}'])
                            .to_string(),
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

pub(super) fn parse_revision_user_names(xml: &str) -> Vec<User> {
    let mut r = Reader::from_str(xml);
    let mut users: Vec<User> = Vec::new();

    loop {
        match r.read_event() {
            Ok(Event::Empty(e)) => {
                if local(e.name().as_ref()) == b"userInfo" {
                    users.push(User {
                        guid: attr(&e, b"guid")
                            .unwrap_or_default()
                            .trim_matches(['{', '}'])
                            .to_string(),
                        name: attr(&e, b"name").unwrap_or_default(),
                        id: attr(&e, b"id")
                            .unwrap_or_default()
                            .parse::<i32>()
                            .unwrap_or_default(),
                        datetime: attr(&e, b"dateTime").unwrap_or_default(),
                    });
                }
            }
            Ok(Event::Eof) | Err(_) => break,
            _ => {}
        }
    }

    users
}

#[allow(clippy::collapsible_match)]
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
                    let sheet_id = attr(&e, b"sId")
                        .and_then(|value| value.parse::<usize>().ok())
                        .unwrap_or_default();
                    if current_revision.is_some() {
                        let current_change = RevisionChangeBuilder::CellChange {
                            sheet_id,
                            changes: Vec::new(),
                        };
                        changes_stack.push(current_change);
                    } else {
                        current_revision = Some(RevisionBuilder::CellChange {
                            revision_id: attr(&e, b"rId")
                                .and_then(|value| value.parse::<usize>().ok())
                                .unwrap_or_default(),
                            sheet_id,
                            changes: Vec::new(),
                        })
                    }
                }

                b"rfmt" => {
                    current_revision = Some(RevisionBuilder::Formatting {
                        sheet_id: attr(&e, b"sheetId")
                            .and_then(|value| value.parse::<usize>().ok())
                            .unwrap_or_default(),
                        start: attr(&e, b"start").and_then(|value| value.parse::<usize>().ok()),
                        length: attr(&e, b"length").and_then(|value| value.parse::<usize>().ok()),
                        address: attr(&e, b"sqref").unwrap_or_default(),
                        formatting: None,
                    });
                }

                b"nc" => {
                    let change = RevisionChangeBuilder::NewCell {
                        value: None,
                        address: attr(&e, b"r").unwrap_or_default(),
                        is_bool: attr(&e, b"t").is_some_and(|value| value == "b"),
                    };
                    changes_stack.push(change);
                }

                b"oc" => {
                    let change = RevisionChangeBuilder::OldCell {
                        value: None,
                        address: attr(&e, b"r").unwrap_or_default(),
                        is_bool: attr(&e, b"t").is_some_and(|value| value == "b"),
                    };
                    changes_stack.push(change);
                }

                b"dxf" => {
                    if let Some(current_revision) = current_revision.as_mut() {
                        if let RevisionBuilder::Formatting { formatting, .. } = current_revision {
                            *formatting = Some(RevisionDifferentialFormat::default());
                        }
                    }
                }

                b"ndxf" => {
                    let change = RevisionChangeBuilder::NewDifferentialFormat(
                        RevisionDifferentialFormat::default(),
                    );
                    changes_stack.push(change);
                }
                b"odxf" => {
                    let change = RevisionChangeBuilder::OldDifferentialFormat(
                        RevisionDifferentialFormat::default(),
                    );
                    changes_stack.push(change);
                }

                b"font" => {
                    parser_state = Some(ParserState::Font);
                    if let Some(current_revision) = current_revision.as_mut() {
                        if let RevisionBuilder::Formatting { formatting, .. } = current_revision {
                            if let Some(formatting) = formatting.as_mut() {
                                formatting.font = Some(RevisionFont::default());
                            }
                        }
                    }
                }

                b"v" => {
                    parser_state = Some(ParserState::V);
                }
                b"t" => {
                    parser_state = Some(ParserState::T);
                }
                b"f" => {
                    parser_state = Some(ParserState::F);
                }

                _ => {}
            },

            Ok(Event::Empty(e)) => match local(e.name().as_ref()) {
                b"rrc" => {
                    let sheet_id = attr(&e, b"sId")
                        .and_then(|value| value.parse::<usize>().ok())
                        .unwrap_or_default();
                    // TODO: probably should throw an error if action is not known
                    let action = attr(&e, b"action")
                        .unwrap_or_default()
                        .try_into()
                        .expect("invalid value for RevisionRowColumnAction");
                    if let Some(current_revision) = current_revision.as_mut() {
                        current_revision.push_change(RevisionChangeBuilder::RowColumn {
                            address: attr(&e, b"ref").unwrap_or_default(),
                            sheet_id,
                            action,
                            changes: Vec::new(),
                        })
                    } else {
                        revisions.push(RevisionBuilder::RowColumn {
                            address: attr(&e, b"ref").unwrap_or_default(),
                            revision_id: attr(&e, b"rId")
                                .and_then(|value| value.parse::<usize>().ok())
                                .unwrap_or_default(),
                            sheet_id,
                            action,
                            changes: Vec::new(),
                            eol: attr(&e, b"eol")
                                .map(|v| v == "1" || v == "true")
                                .unwrap_or(false),
                        });
                    }
                }
                b"ris" => {
                    revisions.push(RevisionBuilder::InsertSheet {
                        revision_id: attr(&e, b"rId")
                            .and_then(|value| value.parse::<usize>().ok())
                            .unwrap_or_default(),
                        sheet_id: attr(&e, b"sheetId")
                            .and_then(|value| value.parse::<usize>().ok())
                            .unwrap_or_default(),
                        name: attr(&e, b"name").map(Into::into).unwrap(),
                        sheet_position: attr(&e, b"sheetPosition")
                            .and_then(|value| value.parse::<usize>().ok())
                            .unwrap_or_default(),
                    });
                }
                b"rsnm" => {
                    revisions.push(RevisionBuilder::RenameSheet {
                        revision_id: attr(&e, b"rId")
                            .and_then(|value| value.parse::<usize>().ok())
                            .unwrap_or_default(),
                        sheet_id: attr(&e, b"sheetId")
                            .and_then(|value| value.parse::<usize>().ok())
                            .unwrap_or_default(),
                        old_name: attr(&e, b"oldName").map(Into::into).unwrap(),
                        new_name: attr(&e, b"newName").map(Into::into).unwrap(),
                    });
                }
                b"rcv" => {
                    revisions.push(RevisionBuilder::RevisionView {
                        guid: attr(&e, b"guid")
                            .unwrap_or_default()
                            .trim_matches(['{', '}'])
                            .to_string(),
                        action: match attr(&e, b"action").unwrap_or_default().as_str() {
                            "add" => RevisionViewAction::Add,
                            "delete" => RevisionViewAction::Delete,
                            _ => RevisionViewAction::Add,
                        },
                    });
                }
                b"nc" => {
                    current_revision.as_mut().unwrap().push_change(
                        RevisionChangeBuilder::NewCell {
                            address: attr(&e, b"r").unwrap_or_default(),
                            value: None,
                            is_bool: attr(&e, b"t").is_some_and(|value| value == "b"),
                        },
                    );
                }

                b"b" => {
                    let is_bold = attr(&e, b"val").is_none_or(|value| value != "0");

                    if let Some(current_revision) = current_revision.as_mut() {
                        if let RevisionBuilder::Formatting { formatting, .. } = current_revision {
                            if let Some(formatting) = formatting.as_mut() {
                                if let Some(font) = formatting.font.as_mut() {
                                    font.bold = Some(is_bold);
                                }
                            }
                        }
                    }
                }

                b"i" => {
                    let is_italic = attr(&e, b"val").is_none_or(|value| value != "0");

                    if let Some(current_revision) = current_revision.as_mut() {
                        if let RevisionBuilder::Formatting { formatting, .. } = current_revision {
                            if let Some(formatting) = formatting.as_mut() {
                                if let Some(font) = formatting.font.as_mut() {
                                    font.italic = Some(is_italic);
                                }
                            }
                        }
                    }
                }

                b"u" => {
                    let is_underline = attr(&e, b"val").is_none_or(|value| value != "none");

                    if let Some(current_revision) = current_revision.as_mut() {
                        if let RevisionBuilder::Formatting { formatting, .. } = current_revision {
                            if let Some(formatting) = formatting.as_mut() {
                                if let Some(font) = formatting.font.as_mut() {
                                    font.underline = Some(is_underline);
                                }
                            }
                        }
                    }
                }

                b"strike" => {
                    let is_strikethrough = attr(&e, b"val").is_none_or(|value| value != "0");

                    if let Some(current_revision) = current_revision.as_mut() {
                        if let RevisionBuilder::Formatting { formatting, .. } = current_revision {
                            if let Some(formatting) = formatting.as_mut() {
                                if let Some(font) = formatting.font.as_mut() {
                                    font.strikethrough = Some(is_strikethrough);
                                }
                            }
                        }
                    }
                }

                b"sz" => {
                    let value = attr(&e, b"val").and_then(|value| value.parse::<f32>().ok());

                    if let Some(current_revision) = current_revision.as_mut() {
                        if let RevisionBuilder::Formatting { formatting, .. } = current_revision {
                            if let Some(formatting) = formatting.as_mut() {
                                if let Some(font) = formatting.font.as_mut() {
                                    font.size = value;
                                }
                            }
                        }
                    }
                }
                b"numFmt" => {
                    let number_format_id = attr(&e, b"numFmtId")
                        .and_then(|value| value.parse::<u16>().ok())
                        .unwrap_or_default();
                    let format_code = attr(&e, b"formatCode").unwrap_or_default();
                    if let Some(current_change) = changes_stack.last_mut() {
                        if let RevisionChangeBuilder::NewDifferentialFormat(formatting)
                        | RevisionChangeBuilder::OldDifferentialFormat(formatting) =
                            current_change
                        {
                            formatting.number_format = Some(RevisionNumberFormat {
                                number_format_id,
                                format_code,
                            });
                        }
                    }
                }

                _ => {}
            },

            Ok(Event::Text(e)) => {
                if let Some(state) = parser_state.as_ref() {
                    if let Some(current_change) = &mut changes_stack.last_mut() {
                        match state {
                            ParserState::V => {
                                if let RevisionChangeBuilder::NewCell { value, is_bool, .. }
                                | RevisionChangeBuilder::OldCell { value, is_bool, .. } =
                                    current_change
                                {
                                    let new_value = text_of(&e);
                                    if *is_bool {
                                        match new_value.parse::<u8>() {
                                            Ok(new_value) => {
                                                match new_value {
                                                    0 => *value = Some(Cell::Bool(false)),
                                                    1 => *value = Some(Cell::Bool(true)),
                                                    _ => todo!(), // TODO: add an error
                                                }
                                            }

                                            Err(_) => todo!(), // TODO: add an error
                                        }
                                    } else if let Ok(new_value) = new_value.parse::<f64>() {
                                        *value = Some(Cell::Number(new_value));
                                    } else {
                                        *value = Some(Cell::Text(new_value));
                                    }
                                }
                            }
                            ParserState::T => {
                                if let RevisionChangeBuilder::NewCell { value, .. }
                                | RevisionChangeBuilder::OldCell { value, .. } = current_change
                                {
                                    *value = Some(Cell::Text(text_of(&e)));
                                }
                            }
                            ParserState::F => {
                                if let RevisionChangeBuilder::NewCell { value, .. }
                                | RevisionChangeBuilder::OldCell { value, .. } = current_change
                                {
                                    *value = Some(Cell::Formula {
                                        formula: text_of(&e),
                                        cached: Box::new(Cell::Text("".to_string())),
                                    });
                                }
                            }
                            _ => {}
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

                b"rfmt" => {
                    if let Some(current_revision) = current_revision.take() {
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

                b"oc" => {
                    if let Some(last_change) =
                        changes_stack.pop_if(|c| matches!(c, RevisionChangeBuilder::OldCell { .. }))
                    {
                        current_revision.as_mut().unwrap().push_change(last_change);
                    }
                }

                b"ndxf" => {
                    if let Some(last_change) = changes_stack.pop_if(|c| {
                        matches!(c, RevisionChangeBuilder::NewDifferentialFormat { .. })
                    }) {
                        current_revision.as_mut().unwrap().push_change(last_change);
                    }
                }
                b"odxf" => {
                    if let Some(last_change) = changes_stack.pop_if(|c| {
                        matches!(c, RevisionChangeBuilder::OldDifferentialFormat { .. })
                    }) {
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
        revision_log_id: revision_ref.rid.clone(),
        user_name: revision_ref.user_name.clone(),
        revisions,
        n_sheets: revision_ref.sheet_ids.len(),
    }
}
