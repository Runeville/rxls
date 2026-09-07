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

#[derive(Debug, Clone)]
pub struct Revision {
    pub revision_id: usize,
    pub user_name: String,
    pub changes: Vec<RevisionChange>,
}
