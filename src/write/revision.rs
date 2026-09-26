use crate::write::xml::{NS_AC, NS_MAIN, NS_MC, NS_PKG_REL, NS_R, REL_REVISION_LOG, XML_DECL};
use crate::{Cell, Revision, RevisionChange, RevisionLog, User};

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
                .min_rid()
                .map(|id| format!(r#"minRId="{id}""#))
                .unwrap_or_default(),
            revision_log
                .max_rid()
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
                push_changes(&mut s, &changes);
                s.push_str("</rcc>");
            }
            Revision::RowColumn {
                rid,
                sid,
                action,
                address,
                changes,
                eol,
            } => {
                if changes.is_empty() {
                    s.push_str(&format!(
                        r#"<rrc rId="{rid}" sId="{sid}" action="{action}" ref="{address}" eol="{}"/>"#, if eol { "1" } else { "0" }
                    ));
                } else {
                    s.push_str(&format!(
                        r#"<rrc rId="{rid}" sId="{sid}" action="{action}" ref="{address}" eol="{}">"#, if eol { "1" } else { "0" }
                    ));
                    push_changes(&mut s, &changes);
                    s.push_str("</rrc>");
                }
            }
            Revision::RevisionView { guid, action } => {
                s.push_str(&format!(
                    r#"<rcv guid="{{{}}} action="{}"/>""#,
                    guid, action
                ));
            }
            Revision::InsertSheet {
                rid,
                sid,
                name,
                sheet_position,
            } => {
                s.push_str(&format!(
                    r#"<ris rId="{}" sheetId="{}" name="{}" sheetPosition="{}"/>"#,
                    rid, sid, name, sheet_position
                ));
            }
            Revision::RenameSheet {
                rid,
                sid,
                old_name,
                new_name,
            } => {
                s.push_str(&format!(
                    r#"<rsnm rId="{}" sheetId="{}" oldName="{}" newName="{}"/>"#,
                    rid, sid, old_name, new_name
                ));
            }
            _ => {}
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

#[allow(clippy::single_match)]
fn push_changes(s: &mut String, changes: &[RevisionChange]) {
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
}
