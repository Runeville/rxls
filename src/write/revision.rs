use crate::write::xml::{NS_AC, NS_MAIN, NS_MC, NS_PKG_REL, NS_R, REL_REVISION_LOG, XML_DECL};
use crate::{Cell, Revision, RevisionChange, RevisionLog};

pub(super) fn revision_headers_xml(revision_logs: &[RevisionLog]) -> String {
    let mut s = String::new();
    s.push_str(XML_DECL);
    s.push_str(&format!(r#"<headers xmlns="{NS_MAIN}" xmlns:r="{NS_R}" xmlns:mc="{NS_MC}" xmlns:x14ac="{NS_AC}" mc:Ignorable="x14ac" guid="{{{}}}" diskRevisions="1" revisionId="{}" version="3">"#, revision_logs.last().unwrap().guid, revision_logs.len()));

    for revision_log in revision_logs {
        let max_sheet_id = revision_log.n_sheets + 1;
        let max_rid = revision_log
            .revisions
            .iter()
            .map(|r| r.id())
            .max()
            .unwrap_or_default();

        s.push_str(&format!(
            r#"<header guid="{{{}}}" dateTime="{}" maxSheetId="{}" userName="{}" r:id="{}" {} {}>"#,
            revision_log.guid,
            revision_log.date_time,
            max_sheet_id + 1,
            revision_log.user_name,
            revision_log.revision_log_id,
            revision_log
                .min_rid
                .map(|id| format!(r#"minRId="{id}""#))
                .unwrap_or_default(),
            max_rid
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
            Revision::CellChange { rid, sid, changes } => {
                s.push_str(&format!(r#"<rcc rId="{}" sId="{}">"#, rid, sid));
                for change in changes {
                    match change {
                        RevisionChange::NewCell { address, value } => {
                            match value {
                                Cell::Text(text) => {
                                    s.push_str(&format!(r#"<nc r="{}" t="inlineStr">"#, address));
                                    s.push_str("<is>");
                                    s.push_str(&format!("<t>{}</t>", text));
                                    s.push_str("</is>");
                                }
                                Cell::Formula { formula, .. } => {
                                    s.push_str(&format!(r#"<nc r="{}">"#, address));
                                    s.push_str(&format!("<f>{}</f>", formula));
                                }
                                Cell::Number(number) => {
                                    s.push_str(&format!(r#"<nc r="{}">"#, address));
                                    s.push_str(&format!("<v>{}</v>", number));
                                }
                                _ => {}
                            }
                            s.push_str("</nc>");
                        }
                        _ => {}
                    }
                }
                s.push_str("</rcc>");
            }
            _ => {}
        }
    }

    if !empty_log {
        s.push_str("</revisions>");
    }
    s
}

pub(super) fn revisions_user_names_xml(revision_logs: &[RevisionLog]) -> String {
    let mut s = String::new();
    s.push_str(XML_DECL);
    s.push_str(&format!(
        r#"<users xmlns="{NS_MAIN}" xmlns:r="{NS_R}" xmlns:mc="{NS_MC}" xmlns:x14ac="{NS_AC}" mc:Ignorable="x14ac" count="0"/>"#
    ));
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
