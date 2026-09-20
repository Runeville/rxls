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
}

impl Revision {
    /// Gets rId of revision if revision has it
    pub fn id(&self) -> Option<usize> {
        match self {
            Revision::RowColumn { rid, .. } => Some(*rid),
            Revision::CellChange { rid, .. } => Some(*rid),
            Revision::Formatting { .. } => None,
        }
    }

    /// Gets sId if revision has it
    pub fn sid(&self) -> Option<usize> {
        match self {
            Revision::RowColumn { sid, .. } => Some(*sid),
            Revision::CellChange { sid, .. } => Some(*sid),
            Revision::Formatting { .. } => None,
        }
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
