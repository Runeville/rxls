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
}

/// Enum that provides different types of revision change
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub enum RevisionChangeEnum {
    /// Insert or delete row or column
    RevisionRowColumn {
        /// Action taken like Insert or Delete
        action: RevisionRowColumnAction,
        /// Address of inserted or deleted item
        address: String,
    },
    /// Change cell
    RevisionCellChange {
        /// New value
        value: Cell,
        /// Address of a changed cell
        address: String,
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

/// One revision inside revisionLog
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct RevisionChange {
    /// Revision id
    pub rid: usize,
    /// Sheet id
    pub sid: usize,
    /// Change type
    pub change: RevisionChangeEnum,
}

/// One revision log
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Revision {
    /// Revision id from r:id
    pub revision_id: String,
    /// User name from userName field
    pub user_name: String,
    /// Revision log's revisions
    pub changes: Vec<RevisionChange>,
}
