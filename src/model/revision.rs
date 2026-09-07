use super::Cell;

#[derive(Debug, Clone)]
pub enum RevisionRowColumnAction {
    InsertRow,
    DeleteRow,
}

#[derive(Debug, Clone)]
pub enum RevisionChangeEnum {
    RevisionRowColumn {
        action: RevisionRowColumnAction,
        address: String,
    },
    RevisionCellChange {
        value: Cell,
        address: String,
    },
    Formatting {
        start: usize,
        length: usize,
        address: String,
        action: String,
    },
}

#[derive(Debug, Clone)]
pub struct RevisionChange {
    index: usize,
    sheet_id: usize,
    change: RevisionChangeEnum,
}

/// One revision log
#[derive(Debug, Clone)]
pub struct Revision {
    /// Revision id from r:id
    pub revision_id: usize,
    /// User name from userName field
    pub user_name: String,
    /// Revision log's revisions
    pub changes: Vec<RevisionChange>,
}

#[derive(Debug, Clone)]
pub struct ParsedRevisionHeaders {
    pub revision_id: usize,
    pub user_name: String,
    pub changes: Vec<RevisionChange>,
}
