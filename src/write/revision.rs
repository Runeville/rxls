use crate::write::xml::{NS_AC, NS_MAIN, NS_MC, NS_PKG_REL, NS_R, REL_REVISION_LOG, XML_DECL};
use crate::{Cell, FormatPattern, Revision, RevisionLog, User};

pub(super) fn revision_headers_xml(revision_logs: &[RevisionLog]) -> String {
    let mut s = String::new();
    s.push_str(XML_DECL);
    s.push_str(&format!(r#"<headers xmlns="{NS_MAIN}" xmlns:r="{NS_R}" xmlns:mc="{NS_MC}" xmlns:x14ac="{NS_AC}" mc:Ignorable="x14ac" guid="{{{}}}" diskRevisions="1" revisionId="{}" version="3">"#, revision_logs.last().unwrap().guid, revision_logs.len()));

    for revision_log in revision_logs {
        let max_sheet_id = revision_log.n_sheets + 1;

        s.push_str(&format!(
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
        ));
        s.push_str(&format!(
            r#"<sheetIdMap count="{}">"#,
            revision_log.n_sheets
        ));
        for i in 1..=revision_log.n_sheets {
            s.push_str(&format!(r#"<sheetId val="{}"/>"#, i));
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

    s.push_str(&format!(r#"<revisions xmlns="{NS_MAIN}" xmlns:r="{NS_R}" xmlns:mc="{NS_MC}" xmlns:x14ac="{NS_AC}" mc:Ignorable="x14ac"{}>"#, if empty_log { "/" } else { "" }));

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

                s.push_str(&format!(
                    r#"<rcc rId="{}" sId="{}"{odxf_s}{dxf_s}{num_fmt_s}>"#,
                    revision_cell_change.revision_id, revision_cell_change.sheet_id,
                ));

                // <oc>
                match &revision_cell_change.old_cell {
                    Some(Cell::Text(text)) => {
                        s.push_str(&format!(r#"<oc r="{}" t="inlineStr">"#, address));
                        s.push_str("<is>");
                        s.push_str(&format!("<t>{}</t>", text));
                        s.push_str("</is>");
                    }
                    Some(Cell::Formula { formula, .. }) => {
                        s.push_str(&format!(r#"<oc r="{}">"#, address));
                        s.push_str(&format!("<f>{}</f>", formula));
                    }
                    Some(Cell::Number(number)) => {
                        s.push_str(&format!(r#"<oc r="{}">"#, address));
                        s.push_str(&format!("<v>{}</v>", number));
                    }
                    Some(Cell::Bool(boolean)) => {
                        s.push_str(&format!(r#"<oc r="{}" t="b">"#, address));
                        s.push_str(&format!("<v>{}</v>", if *boolean { "1" } else { "0" }));
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
                            s.push_str(&format!(r#"<nc r="{address}" t="inlineStr">"#));
                            s.push_str("<is>");
                            s.push_str(&format!("<t>{}</t>", text));
                            s.push_str("</is>");
                        }
                        Cell::Formula { formula, .. } => {
                            s.push_str(&format!(r#"<nc r="{address}">"#));
                            s.push_str(&format!("<f>{}</f>", formula));
                        }
                        Cell::Number(number) => {
                            s.push_str(&format!(r#"<nc r="{address}">"#));
                            s.push_str(&format!("<v>{}</v>", number));
                        }
                        Cell::Bool(boolean) => {
                            s.push_str(&format!(r#"<nc r="{address}" t="b">"#));
                            s.push_str(&format!("<v>{}</v>", if *boolean { "1" } else { "0" }));
                        }
                        _ => {}
                    },
                    None => {
                        s.push_str(&format!(r#"<nc r="{}"/>"#, address));
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
                            s.push_str(&format!(
                                r#"<numFmt numFmtId="{}" formatCode="{}"/>"#,
                                num_fmt.number_format_id, num_fmt.format_code
                            ));
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
                            s.push_str(&format!(
                                r#"<numFmt numFmtId="{}" formatCode="{}"/>"#,
                                number_format.number_format_id, number_format.format_code
                            ));
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
                    s.push_str(&format!(
                        r#"<rrc rId="{revision_id}" sId="{sheet_id}" action="{action}" ref="{address}" eol="{}"/>"#, if eol { "1" } else { "0" }
                    ));
                } else {
                    s.push_str(&format!(
                        r#"<rrc rId="{revision_id}" sId="{sheet_id}" action="{action}" ref="{address}" eol="{}">"#, if eol { "1" } else { "0" }
                    ));
                    todo!("RevisionChanges restructure");
                    s.push_str("</rrc>");
                }
            }
            Revision::RevisionView { guid, action } => {
                s.push_str(&format!(
                    r#"<rcv guid="{{{}}}" action="{}"/>"#,
                    guid, action
                ));
            }
            Revision::InsertSheet {
                revision_id,
                sheet_id,
                name,
                sheet_position,
            } => {
                s.push_str(&format!(
                    r#"<ris rId="{}" sheetId="{}" name="{}" sheetPosition="{}"/>"#,
                    revision_id, sheet_id, name, sheet_position
                ));
            }
            Revision::RenameSheet {
                revision_id,
                sheet_id,
                old_name,
                new_name,
            } => {
                s.push_str(&format!(
                    r#"<rsnm rId="{}" sheetId="{}" oldName="{}" newName="{}"/>"#,
                    revision_id, sheet_id, old_name, new_name
                ));
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

                s.push_str(&format!(
                    r#"<rfmt sheetId="{sheet_id}" sqref="{address}"{start}{length}>"#
                ));

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
                            s.push_str(&format!(r#"<sz val="{}"/>"#, size));
                        }
                        s.push_str("</font>");
                    }

                    if let Some(number_format) = formatting.number_format {
                        s.push_str(&format!(
                            r#"<numFmt numFmtId="{}" formatCode="{}"/>"#,
                            number_format.number_format_id, number_format.format_code
                        ));
                    }

                    if let Some(fill) = formatting.fill {
                        let pattern = match fill.pattern {
                            FormatPattern::Solid => "solid",
                            _ => "none",
                        };
                        s.push_str(&format!(r#"<patternFill patternType="{pattern}">"#));

                        if let Some(background) = fill.background {
                            let theme = background
                                .theme
                                .map_or(String::new(), |t| format!(r#" theme="{t}""#));

                            let tint = background
                                .tint
                                .map_or(String::new(), |t| format!(r#" tint="{t}""#));
                            let rgb = background.rgb.map_or(String::new(), |[r, g, b]| {
                                format!(r#"rgb="FF{r:02X}{g:02X}{b:02X}""#)
                            });

                            s.push_str(&format!(r"<bgColor{theme}{tint}{rgb}/>"));
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
                                format!(r#"rgb="FF{r:02X}{g:02X}{b:02X}""#)
                            });

                            s.push_str(&format!(r"<fgColor{theme}{tint}{rgb}/>"));
                        }

                        s.push_str("</patternFill>");
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
        s.push_str(&format!(
        r#"<users xmlns="{NS_MAIN}" xmlns:r="{NS_R}" xmlns:mc="{NS_MC}" xmlns:x14ac="{NS_AC}" mc:Ignorable="x14ac" count="0"/>"#
    ));
        return s;
    }

    s.push_str(&format!(
        r#"<users xmlns="{NS_MAIN}" xmlns:r="{NS_R}" xmlns:mc="{NS_MC}" xmlns:x14ac="{NS_AC}" mc:Ignorable="x14ac" count="{}">"#,
        users.len()
    ));

    for user in users {
        s.push_str(&format!(
            r#"<userInfo guid="{{{}}}" name="{}" id="{}" dateTime="{}"/>"#,
            user.guid, user.name, user.id, user.datetime
        ));
    }
    s.push_str("</users>");

    s
}

pub(super) fn revision_headers_rels(n_revision_logs: usize) -> String {
    let mut s = String::new();
    s.push_str(XML_DECL);
    s.push_str(&format!(r#"<Relationships xmlns="{NS_PKG_REL}">"#));
    for i in 0..n_revision_logs {
        s.push_str(&format!(
            r#"<Relationship Id="rId{}" Type="{REL_REVISION_LOG}" Target="revisionLog{}.xml"/>"#,
            i + 1,
            i + 1
        ));
    }
    s.push_str("</Relationships>");
    s
}
