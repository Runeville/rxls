use std::fmt;

#[cfg(feature = "serde")]
use serde::Serialize;

use super::{Cell, FormatPattern};

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

impl TryFrom<&str> for RevisionRowColumnAction {
    type Error = &'static str;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        match s {
            "insertRow" => Ok(Self::InsertRow),
            "deleteRow" => Ok(Self::DeleteRow),
            "insertCol" => Ok(Self::InsertCol),
            "deleteCol" => Ok(Self::DeleteCol),
            _ => Err("invalid value for RevisionRowColumnAction"),
        }
    }
}

impl TryFrom<String> for RevisionRowColumnAction {
    type Error = &'static str;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        Self::try_from(s.as_str())
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
#[cfg_attr(feature = "serde", serde(tag = "type"))]
pub enum RevisionChange {
    /// <rcc> cell change
    CellChange(RevisionCellChange),
    /// <rrc> row/column change
    RowColumn {
        /// sId
        sheet_id: usize,
        /// Action taken like Insert or Delete
        action: RevisionRowColumnAction,
        /// Address of inserted or deleted item
        address: String,
        /// Changes
        changes: Vec<RevisionChange>,
    },
    /// <ndxf> new differential format
    NewDifferentialFormat(DifferentialFormat),
    /// <odxf> old differential format
    OldDifferentialFormat(DifferentialFormat),
}

/// One revision inside revisionLog
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[cfg_attr(feature = "serde", serde(tag = "type"))]
pub enum Revision {
    /// Insert or delete row or column
    RowColumn {
        /// rId
        revision_id: usize,
        /// sId
        sheet_id: usize,
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
    CellChange(RevisionCellChange),
    /// Format cell
    Formatting {
        /// sId
        sheet_id: usize,
        /// Start indicates where to apply to apply the formatting on the string
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        start: Option<usize>,
        /// Length indicates where to apply to apply the formatting on the string
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        length: Option<usize>,
        /// Address of formatted cell
        address: String,
        /// Action type
        #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
        formatting: Option<DifferentialFormat>,
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
        revision_id: usize,
        /// sId
        sheet_id: usize,
        /// Sheet name
        name: SheetName,
        /// Sheet position
        sheet_position: usize,
    },
    /// Rename sheet
    /// example: <rsnm rId="5" sheetId="3" oldName="[test.xlsx]Sheet1" newName="[test.xlsx]hello"/>
    RenameSheet {
        /// rId
        revision_id: usize,
        /// sId
        sheet_id: usize,
        /// Old sheet name
        old_name: SheetName,
        /// New sheet name
        new_name: SheetName,
    },
}

/// <rcc> revision used to change cell value (may contain formatting, but value is always present)
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct RevisionCellChange {
    /// rId
    pub revision_id: usize,
    /// sId
    pub sheet_id: usize,
    /// address
    pub address: String,
    /// <nc>
    pub new_cell: Option<Cell>,
    /// <oc>
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub old_cell: Option<Cell>,
    /// <odxf>
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub old_differential_format: Option<DifferentialFormat>,
    /// <ndxf>
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub new_differential_format: Option<DifferentialFormat>,
}

impl Revision {
    /// Gets rId of revision if revision has it
    pub fn id(&self) -> Option<usize> {
        match self {
            Revision::RowColumn { revision_id, .. } => Some(*revision_id),
            Revision::CellChange(cell_change) => Some(cell_change.revision_id),
            Revision::Formatting { .. } | Revision::RevisionView { .. } => None,
            Revision::InsertSheet { revision_id, .. } => Some(*revision_id),
            Revision::RenameSheet { revision_id, .. } => Some(*revision_id),
        }
    }

    /// Gets sId (sheet id) if revision has it
    pub fn sheet_id(&self) -> Option<usize> {
        match self {
            Revision::RowColumn { sheet_id, .. }
            | Revision::InsertSheet { sheet_id, .. }
            | Revision::RenameSheet { sheet_id, .. }
            | Revision::Formatting { sheet_id, .. } => Some(*sheet_id),
            Revision::CellChange(cell_change) => Some(cell_change.sheet_id),
            Revision::RevisionView { .. } => None,
        }
    }
}

/// Sheet name used in some revision types
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct SheetName {
    /// Name of the workbook
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub workbook_name: Option<String>,
    /// Name of the sheet
    pub name: String,
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
    pub fn max_revision_id(&self) -> Option<usize> {
        let max_rid = self
            .revisions
            .iter()
            .map(|r| r.id())
            .max()
            .unwrap_or_default();

        max_rid
    }

    /// Gets min revision id of revision log
    pub fn min_revision_id(&self) -> Option<usize> {
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

/// <rfmt> formatting
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct DifferentialFormat {
    /// <font> font formatting
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub font: Option<Font>,
    /// <numFmt> number format
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub number_format: Option<NumberFormat>,
    /// <fill> fill formatting
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub fill: Option<Fill>,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// <fill> fill formatting
pub struct Fill {
    /// <patternFill> pattern fill
    pub pattern: FormatPattern,
    /// <bgColor> background color
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub background: Option<Color>,
    /// <fgColor> foreground color
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub foreground: Option<Color>,
}

#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
/// bgColor and fgColor props
pub struct Color {
    /// theme attribute
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub theme: Option<u8>,
    /// tint attribute
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub tint: Option<f64>,
    /// rgb attribute
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub rgb: Option<[u8; 3]>,
    /// auto attribute
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub auto: Option<bool>,
}

/// <numFmt> number format
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct NumberFormat {
    /// numFmtId
    pub number_format_id: u16,
    /// formatCode
    pub format_code: String,
}

impl NumberFormat {
    /// Try to create a NumberFormat from a format code
    pub fn try_from_format_code(format_code: String) -> Option<Self> {
        Some(Self {
            number_format_id: match format_code.as_str() {
                // General
                "General" => Some(0),

                // Number
                "0" => Some(1),
                "0.00" => Some(2),
                "#,##0" => Some(3),
                "#,##0.00" => Some(4),

                // Currency
                r##""$"#,##0_);("$"#,##0)"## => Some(5),
                r##""$"#,##0_);[Red]("$"#,##0)"## => Some(6),
                r##""$"#,##0.00_);("$"#,##0.00)"## => Some(7),
                r##""$"#,##0.00_);[Red]("$"#,##0.00)"## => Some(8),

                // Percentage
                "0%" => Some(9),
                "0.00%" => Some(10),

                // Scientific
                "0.00E+00" => Some(11),

                // Fraction
                "# ?/?" => Some(12),
                "# ??/??" => Some(13),

                // Date
                "mm-dd-yy" | "m/d/yyyy" => Some(14),
                "d-mmm-yy" => Some(15),
                "d-mmm" => Some(16),
                "mmm-yy" => Some(17),

                // Time
                "h:mm AM/PM" => Some(18),
                "h:mm:ss AM/PM" => Some(19),
                "h:mm" => Some(20),
                "h:mm:ss" => Some(21),

                // Date and time
                "m/d/yy h:mm" | "m/d/yyyy h:mm" => Some(22),

                // Number with negative formatting
                "#,##0 ;(#,##0)" | "#,##0_);(#,##0)" => Some(37),
                "#,##0 ;[Red](#,##0)" | "#,##0_);[Red](#,##0)" => Some(38),
                "#,##0.00;(#,##0.00)" | "#,##0.00_);(#,##0.00)" => Some(39),
                "#,##0.00;[Red](#,##0.00)" | "#,##0.00_);[Red](#,##0.00)" => Some(40),

                // Accounting
                r#"_(* #,##0_);_(* \(#,##0\);_(* "-"_);_(@_)"# => Some(41),
                r#"_("$"* #,##0_);_("$"* \(#,##0\);_("$"* "-"_);_(@_)"# => Some(42),
                r#"_(* #,##0.00_);_(* \(#,##0.00\);_(* "-"??_);_(@_)"# => Some(43),
                r#"_("$"* #,##0.00_);_("$"* \(#,##0.00\);_("$"* "-"??_);_(@_)"# => Some(44),

                // Time
                "mm:ss" => Some(45),
                "[h]:mm:ss" => Some(46),
                "mmss.0" | "mm:ss.0" => Some(47),

                // Scientific
                "##0.0E+0" => Some(48),

                // Text
                "@" => Some(49),

                _ => None,
            }?,
            format_code,
        })
    }
}

/// <font> possible inner tags
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize))]
pub struct Font {
    /// <b> bold
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub bold: Option<bool>,
    /// <i> italic
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub italic: Option<bool>,
    /// <strike> strikethrough
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub strikethrough: Option<bool>,
    /// <u> underline
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub underline: Option<bool>,
    /// <sz> font size
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub size: Option<f32>,
}
