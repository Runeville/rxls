use std::{fmt, str::FromStr};

#[cfg(feature = "serde")]
use serde::Serialize;

use super::Cell;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// All the data of revisions
pub struct RevisionData {
    /// All revisions logs
    pub revision_logs: Vec<RevisionLog>,
    /// All users sessions
    pub users: Vec<User>,
}

/// Enum for `RevisionChangeEnum::RevisionRowColumn.action`
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum RevisionRowColumnAction {
    /// InsertRow
    InsertRow,
    /// DeleteRow
    DeleteRow,
    /// InsertColumn
    InsertCol,
    /// DeleteColumn
    DeleteCol,
}

impl fmt::Display for RevisionRowColumnAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::InsertRow => "insertRow",
            Self::DeleteRow => "deleteRow",
            Self::InsertCol => "insertCol",
            Self::DeleteCol => "deleteCol",
        };

        f.write_str(value)
    }
}

/// Enum for `RevisionChangeEnum::RevisionView.action`
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum RevisionViewAction {
    /// Add
    Add,
    /// Delete
    Delete,
}

impl fmt::Display for RevisionViewAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Add => "add",
            Self::Delete => "delete",
        };

        f.write_str(value)
    }
}

/// In revisionLog this
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum RevisionChange {
    /// <nc> new cell
    NewCell {
        /// ref
        address: String,
        /// <v> value
        value: Cell,
    },
    /// <rcc> cell change
    CellChange {
        /// sid
        sid: usize,
        /// changes
        changes: Vec<RevisionChange>,
    },
    /// <rrc> row/column change
    RowColumn {
        /// sId
        sid: usize,
        /// Action taken like Insert or Delete
        action: RevisionRowColumnAction,
        /// Address of inserted or deleted item
        address: String,
        /// Changes
        changes: Vec<RevisionChange>,
    },
}

/// One revision inside revisionLog
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum Revision {
    /// Insert or delete row or column
    RowColumn {
        /// rId
        rid: usize,
        /// sId
        sid: usize,
        /// Action taken like Insert or Delete
        action: RevisionRowColumnAction,
        /// Address of inserted or deleted item
        address: String,
        /// Changes
        changes: Vec<RevisionChange>,
        /// End of list
        eol: bool,
    },
    /// Change cell
    CellChange {
        /// rId
        rid: usize,
        /// sId
        sid: usize,
        /// Changes
        changes: Vec<RevisionChange>,
    },
    /// Format cell
    Formatting {
        /// Start indicates where to apply to apply the formatting on the string
        start: usize,
        /// Length indicates where to apply to apply the formatting on the string
        length: usize,
        /// Address of formatted cell
        address: String,
        /// Action type
        action: String,
    },
    /// <rcv> revision custom view changes
    RevisionView {
        /// guid of custom view
        guid: String,
        /// Action taken like Add or Delete
        action: RevisionViewAction,
    },
    /// <ris> insert sheet
    /// example: <ris rId="3" sheetId="2" name="[test.xlsx]Sheet1" sheetPosition="1"/>
    InsertSheet {
        /// rId
        rid: usize,
        /// sId
        sid: usize,
        /// Sheet name
        name: SheetName,
        /// Sheet position
        sheet_position: usize,
    },
}

impl Revision {
    /// Gets rId of revision if revision has it
    pub fn id(&self) -> Option<usize> {
        match self {
            Revision::RowColumn { rid, .. } => Some(*rid),
            Revision::CellChange { rid, .. } => Some(*rid),
            Revision::Formatting { .. } | Revision::RevisionView { .. } => None,
            Revision::InsertSheet { rid, .. } => Some(*rid),
        }
    }

    /// Gets sId if revision has it
    pub fn sid(&self) -> Option<usize> {
        match self {
            Revision::RowColumn { sid, .. } => Some(*sid),
            Revision::CellChange { sid, .. } => Some(*sid),
            Revision::Formatting { .. } | Revision::RevisionView { .. } => None,
            Revision::InsertSheet { sid, .. } => Some(*sid),
        }
    }
}

/// Sheet name used in some revision types
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct SheetName {
    /// Name of the workbook
    workbook_name: Option<String>,
    /// Name of the sheet
    name: String,
}

impl fmt::Display for SheetName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.workbook_name {
            Some(workbook_name) => write!(f, "[{}]{}", workbook_name, self.name),
            None => write!(f, "{}", self.name),
        }
    }
}

impl From<&str> for SheetName {
    fn from(s: &str) -> Self {
        if let Some(rest) = s.strip_prefix('[') {
            if let Some((workbook, sheet)) = rest.split_once(']') {
                return Self {
                    workbook_name: Some(workbook.to_string()),
                    name: sheet.to_string(),
                };
            }
        }

        Self {
            workbook_name: None,
            name: s.to_string(),
        }
    }
}

impl From<String> for SheetName {
    fn from(s: String) -> Self {
        Self::from(s.as_str())
    }
}

/// One revision log
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct RevisionLog {
    /// guid
    pub guid: String,
    /// Revision id from r:id
    pub revision_log_id: String,
    /// User name from userName field
    pub user_name: String,
    /// Revision log's revisions
    pub revisions: Vec<Revision>,
    /// Revision log's date/time
    pub date_time: String,
    /// Number of sheets that are affected by this revision log (or just exist while revision log
    /// was taken)
    pub n_sheets: usize,
}

impl RevisionLog {
    /// Gets max revision id of revision log
    pub fn max_rid(&self) -> Option<usize> {
        let max_rid = self
            .revisions
            .iter()
            .map(|r| r.id())
            .max()
            .unwrap_or_default();

        max_rid
    }

    /// Gets min revision id of revision log
    pub fn min_rid(&self) -> Option<usize> {
        let min_rid = self
            .revisions
            .iter()
            .map(|r| r.id())
            .min()
            .unwrap_or_default();

        min_rid
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// One user sessions stored in /xl/revisions/userNames.xml (exists if workbook is shared)
pub struct User {
    /// guid of last synced RevisionLog
    pub guid: String,
    /// name of the user
    pub name: String,
    /// id of the user session
    pub id: i32,
    /// datetime of the user session
    pub datetime: String,
}
