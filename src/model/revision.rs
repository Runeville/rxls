#[cfg(feature = "serde")]
use serde::Serialize;

use super::Cell;

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
    /// Revision log's min rid
    pub min_rid: Option<usize>,
    /// Revision log's max rid
    pub max_rid: Option<usize>,
}
