use std::fmt::Write;

use crate::write::xml::{NS_AC, NS_MAIN, NS_MC, NS_PKG_REL, NS_R, REL_REVISION_LOG, XML_DECL};
use crate::{Cell, FormatPattern, Revision, RevisionLog, User};

pub(super) fn revision_headers_xml(revision_logs: &[RevisionLog]) -> String {
    let mut s = String::new();
    s.push_str(XML_DECL);
    write!(
        s, 
        r#"<headers xmlns="{NS_MAIN}" xmlns:r="{NS_R}" xmlns:mc="{NS_MC}" xmlns:x14ac="{NS_AC}" mc:Ignorable="x14ac" guid="{{{}}}" diskRevisions="1" revisionId="{}" version="3">"#,
        revision_logs.last().unwrap().guid,
        revision_logs.len()
    ).unwrap();

    for revision_log in revision_logs {
        let max_sheet_id = revision_log.n_sheets + 1;
        write!(
            s,
            r#"<header guid="{{{}}}" dateTime="{}" maxSheetId="{}" userName="{}" r:id="{}" {} {}>"#,
            revision_log.guid,
            revision_log.date_time,
            max_sheet_id,
            revision_log.user_name,
            revision_log.revision_log_id,
            revision_log
                .min_revision_id()
                .map(|id| format!(r#"minRId="{id}""#))
                .unwrap_or_default(),
            revision_log
                .max_revision_id()
                .map(|id| format!(r#"maxRId="{id}""#))
                .unwrap_or_default()
        )
        .unwrap();
        write!(s, r#"<sheetIdMap count="{}">"#, revision_log.n_sheets).unwrap();
        for i in 1..=revision_log.n_sheets {
            write!(s, r#"<sheetId val="{i}"/>"#).unwrap();
        }
        s.push_str("</sheetIdMap>");
        s.push_str("</header>");
    }

    s.push_str("</headers>");
    s
}

#[allow(clippy::single_match)]
pub(super) fn revision_log_xml(revision_log: &RevisionLog) -> String {
    let mut s = String::new();
    s.push_str(XML_DECL);
    let empty_log = revision_log.revisions.is_empty();

    write!(s, r#"<revisions xmlns="{NS_MAIN}" xmlns:r="{NS_R}" xmlns:mc="{NS_MC}" xmlns:x14ac="{NS_AC}" mc:Ignorable="x14ac"{}>"#, if empty_log { "/" } else { "" }).unwrap();

    for revision in revision_log.revisions.clone() {
        match revision {
            Revision::CellChange(revision_cell_change) => {
                let address = revision_cell_change.address.as_str();
                let odxf_s = revision_cell_change
                    .old_differential_format
                    .as_ref()
                    .map_or("", |_| r#" odxf="1""#);

                let dxf_s = revision_cell_change
                    .new_differential_format
                    .as_ref()
                    .map_or("", |_| r#" dxf="1""#);

                let num_fmt_s = revision_cell_change
                    .new_differential_format
                    .as_ref()
                    .and_then(|format| format.number_format.as_ref())
                    .map_or(String::new(), |num_fmt| {
                        format!(r#" numFmtId="{}""#, num_fmt.number_format_id)
                    });

                write!(
                    s,
                    r#"<rcc rId="{}" sId="{}"{odxf_s}{dxf_s}{num_fmt_s}>"#,
                    revision_cell_change.revision_id,
                    revision_cell_change.sheet_id
                ).unwrap();

                // <oc>
                match &revision_cell_change.old_cell {
                    Some(Cell::Text(text)) => {
                        write!(s, r#"<oc r="{address}" t="inlineStr">"#).unwrap();
                        s.push_str("<is>");
                        write!(s, "<t>{text}</t>").unwrap();
                        s.push_str("</is>");
                    }
                    Some(Cell::Formula { formula, .. }) => {
                        write!(s, r#"<oc r="{address}">"#).unwrap();
                        write!(s, "<f>{formula}</f>").unwrap();
                    }
                    Some(Cell::Number(number)) => {
                        write!(s, r#"<oc r="{address}">"#).unwrap();
                        write!(s, "<v>{number}</v>").unwrap();
                    }
                    Some(Cell::Bool(boolean)) => {
                        write!(s, r#"<oc r="{address}" t="b">"#).unwrap();
                        write!(s, "<v>{}</v>", if *boolean { "1" } else { "0" }).unwrap();
                    }
                    _ => {}
                }
                if revision_cell_change.old_cell.is_some() {
                    s.push_str("</oc>");
                }

                // <nc>
                match &revision_cell_change.new_cell {
                    Some(value) => match value {
                        Cell::Text(text) => {
                            write!(s, r#"<nc r="{address}" t="inlineStr">"#).unwrap();
                            s.push_str("<is>");
                            write!(s, "<t>{text}</t>").unwrap();
                            s.push_str("</is>");
                        }
                        Cell::Formula { formula, .. } => {
                            write!(s, r#"<nc r="{address}">"#).unwrap();
                            write!(s, "<f>{formula}</f>").unwrap();
                        }
                        Cell::Number(number) => {
                            write!(s, r#"<nc r="{address}">"#).unwrap();
                            write!(s, "<v>{number}</v>").unwrap();
                        }
                        Cell::Bool(boolean) => {
                            write!(s, r#"<nc r="{address}" t="b">"#).unwrap();
                            write!(s, "<v>{}</v>", if *boolean { "1" } else { "0" }).unwrap();
                        }
                        _ => {}
                    },
                    None => {
                        write!(s, r#"<nc r="{address}"/>"#).unwrap();
                    }
                }
                if revision_cell_change.new_cell.is_some() {
                    s.push_str("</nc>");
                }

                // <odxf>
                match &revision_cell_change.old_differential_format {
                    Some(format) => {
                        s.push_str("<odxf>");
                        if let Some(num_fmt) = &format.number_format.as_ref() {
                            write!(s, r#"<numFmt numFmtId="{}" formatCode="{}"/>"#, num_fmt.number_format_id, num_fmt.format_code).unwrap();
                        }
                        s.push_str("</odxf>");
                    }
                    None => {}
                }

                // <ndxf>
                match &revision_cell_change.new_differential_format {
                    Some(format) => {
                        s.push_str("<ndxf>");
                        if let Some(number_format) = &format.number_format {
                            write!(s, r#"<numFmt numFmtId="{}" formatCode="{}"/>"#, number_format.number_format_id, number_format.format_code).unwrap();
                        }
                        s.push_str("</ndxf>");
                    }
                    None => {}
                }

                s.push_str("</rcc>");
            }
            Revision::RowColumn {
                revision_id,
                sheet_id,
                action,
                address,
                changes,
                eol,
            } => {
                if changes.is_empty() {
                    write!(s, r#"<rrc rId="{revision_id}" sId="{sheet_id}" action="{action}" ref="{address}" eol="{}"/>"#, if eol { "1" } else { "0" }).unwrap();
                } else {
                    write!(s, r#"<rrc rId="{revision_id}" sId="{sheet_id}" action="{action}" ref="{address}" eol="{}">"#, if eol { "1" } else { "0" }).unwrap();
                    todo!("RevisionChanges restructure");
                    s.push_str("</rrc>");
                }
            }
            Revision::RevisionView { guid, action } => {
                write!(s, r#"<rcv guid="{{{guid}}}" action="{action}"/>"#).unwrap();
            }
            Revision::InsertSheet {
                revision_id,
                sheet_id,
                name,
                sheet_position,
            } => {
                write!(s, r#"<ris rId="{revision_id}" sheetId="{sheet_id}" name="{name}" sheetPosition="{sheet_position}"/>"#).unwrap();
            }
            Revision::RenameSheet {
                revision_id,
                sheet_id,
                old_name,
                new_name,
            } => {
                write!(s, r#"<rsnm rId="{revision_id}" sheetId="{sheet_id}" oldName="{old_name}" newName="{new_name}"/>"#).unwrap();
            }
            Revision::Formatting {
                sheet_id,
                start,
                length,
                address,
                formatting,
            } => {
                let start = start
                    .map(|v| format!(r#" start="{v}""#))
                    .unwrap_or_default();

                let length = length
                    .map(|v| format!(r#" length="{v}""#))
                    .unwrap_or_default();

                write!(s, r#"<rfmt sheetId="{sheet_id}" sqref="{address}"{start}{length}>"#).unwrap();

                if let Some(formatting) = formatting {
                    s.push_str("<dxf>");

                    if let Some(font) = formatting.font {
                        s.push_str("<font>");
                        if let Some(bold) = font.bold {
                            if bold {
                                s.push_str("<b/>");
                            } else {
                                s.push_str(r#"<b val="0"/>"#);
                            }
                        }
                        if let Some(italic) = font.italic {
                            if italic {
                                s.push_str("<i/>");
                            } else {
                                s.push_str(r#"<i val="0"/>"#);
                            }
                        }
                        if let Some(underline) = font.underline {
                            if underline {
                                s.push_str("<u/>");
                            } else {
                                s.push_str(r#"<u val="none"/>"#);
                            }
                        }
                        if let Some(strike) = font.strikethrough {
                            if strike {
                                s.push_str("<strike/>");
                            } else {
                                s.push_str(r#"<strike val="0"/>"#);
                            }
                        }
                        if let Some(size) = font.size {
                            write!(s, r#"<sz val="{size}"/>"#).unwrap();
                        }
                        if let Some(color) = font.color {
                            let theme = color
                                .theme
                                .map_or(String::new(), |t| format!(r#" theme="{t}""#));

                            let tint = color
                                .tint
                                .map_or(String::new(), |t| format!(r#" tint="{t}""#));

                            let rgb = color.rgb.map_or(String::new(), |[r, g, b]| {
                                format!(r#" rgb="FF{r:02X}{g:02X}{b:02X}""#)
                            });

                            let indexed = color.indexed.map_or(String::new(), |i| {
                                format!(r#" indexed="{i}""#)
                            });

                            let auto = color.auto.map_or(String::new(), |a| {
                                format!(r#" auto="{}""#, if a { "1" } else { "0" })
                            });

                            write!(s, r#"<color{theme}{rgb}{indexed}{tint}{auto}/>"#).unwrap();
                        }

                        if let Some(name) = font.name {
                            write!(s, r#"<name val="{}"/>"#, name).unwrap();
                        }

                        if let Some(family) = font.family {
                            write!(s, r#"<family val="{}"/>"#, family).unwrap();
                        }

                        if let Some(scheme) = font.scheme {
                            write!(s, r#"<scheme val="{}"/>"#, scheme).unwrap();
                        }

                        s.push_str("</font>");
                    }

                    if let Some(number_format) = formatting.number_format {
                        write!(s, r#"<numFmt numFmtId="{}" formatCode="{}"/>"#, number_format.number_format_id, number_format.format_code).unwrap();
                    }

                    if let Some(fill) = formatting.fill {
                        let pattern = match fill.pattern {
                            FormatPattern::Solid => "solid",
                            _ => "none",
                        };
                        s.push_str("<fill>");
                        write!(s, r#"<patternFill patternType="{pattern}">"#).unwrap();

                        if let Some(background) = fill.background {
                            let theme = background
                                .theme
                                .map_or(String::new(), |t| format!(r#" theme="{t}""#));

                            let tint = background
                                .tint
                                .map_or(String::new(), |t| format!(r#" tint="{t}""#));

                            let rgb = background.rgb.map_or(String::new(), |[r, g, b]| {
                                format!(r#" rgb="FF{r:02X}{g:02X}{b:02X}""#)
                            });

                            let auto = background.auto.map_or(String::new(), |a| {
                                format!(r#" auto="{}""#, if a { "1" } else { "0" })
                            });

                            write!(s, r"<bgColor{theme}{tint}{rgb}{auto}/>").unwrap();
                        }
                        if let Some(foreground) = fill.foreground {
                            let theme = foreground
                                .theme
                                .map(|t| format!(r#" theme="{t}""#))
                                .unwrap_or_default();

                            let tint = foreground
                                .tint
                                .map(|t| format!(r#" tint="{t}""#))
                                .unwrap_or_default();

                            let rgb = foreground.rgb.map_or(String::new(), |[r, g, b]| {
                                format!(r#" rgb="FF{r:02X}{g:02X}{b:02X}""#)
                            });

                            let auto = foreground.auto.map_or(String::new(), |a| {
                                format!(r#" auto="{}""#, if a { "1" } else { "0" })
                            });

                            write!(s, r"<fgColor{theme}{tint}{rgb}{auto}/>").unwrap();
                        }

                        s.push_str("</patternFill>");
                        s.push_str("</fill>");
                    }

                    if let Some(alignment) = formatting.alignment.as_ref() {
                        let horizontal = alignment.horizontal.map_or(String::new(), |h| {
                            format!(r#" horizontal="{h}""#)
                        });
                        let vertical = alignment.vertical.map_or(String::new(), |v| {
                            format!(r#" vertical="{v}""#)
                        });

                        // s.push_str("<alignment{horizontal}{vertical}/>");
                        write!(s, "<alignment{vertical}{horizontal}/>").unwrap();
                    }

                    s.push_str("</dxf>");
                }

                s.push_str("</rfmt>");
            }
        }
    }

    if !empty_log {
        s.push_str("</revisions>");
    }
    s
}

pub(super) fn revisions_user_names_xml(users: &[User]) -> String {
    let mut s = String::new();
    s.push_str(XML_DECL);

    if users.is_empty() {
        write!(s, r#"<users xmlns="{NS_MAIN}" xmlns:r="{NS_R}" xmlns:mc="{NS_MC}" xmlns:x14ac="{NS_AC}" mc:Ignorable="x14ac" count="0"/>"#).unwrap();
        return s;
    }

    write!(s, r#"<users xmlns="{NS_MAIN}" xmlns:r="{NS_R}" xmlns:mc="{NS_MC}" xmlns:x14ac="{NS_AC}" mc:Ignorable="x14ac" count="{}">"#, users.len()).unwrap();

    for user in users {
        write!(s, r#"<userInfo guid="{{{}}}" name="{}" id="{}" dateTime="{}"/>"#, user.guid, user.name, user.id, user.datetime).unwrap();
    }
    s.push_str("</users>");

    s
}

pub(super) fn revision_headers_rels(n_revision_logs: usize) -> String {
    let mut s = String::new();
    s.push_str(XML_DECL);
    write!(s, r#"<Relationships xmlns="{NS_PKG_REL}">"#).unwrap();
    for i in 0..n_revision_logs {
        write!(s, r#"<Relationship Id="rId{}" Type="{REL_REVISION_LOG}" Target="revisionLog{}.xml"/>"#, i + 1, i + 1).unwrap();
    }
    s.push_str("</Relationships>");
    s
}
