use crate::write::xml::{NS_AC, NS_MAIN, NS_MC, NS_PKG_REL, NS_R, REL_REVISION_LOG, XML_DECL};
use crate::RevisionLog;

pub(super) fn revision_headers_xml(revision_logs: &[RevisionLog]) -> String {
    let mut s = String::new();
    s.push_str(XML_DECL);
    for revision_log in revision_logs {
        let max_sheet_id = revision_log
            .revisions
            .iter()
            .map(|r| r.sid().unwrap_or_default())
            .max()
            .unwrap_or_default();

        s.push_str(&format!(
            r#"<header guid="{{{}}}" dateTime="{}" maxSheetId="{}" userName="{}" r:id="{}" minRId="{}">"#,
            revision_log.guid,
            revision_log.date_time,
            "todo",
            revision_log.user_name,
            revision_log.revision_log_id,
            revision_log.min_rid.unwrap_or_default()
        ));
        s.push_str(&format!(r#"<sheetIdMap count="{}">"#, max_sheet_id));
        for i in 1..=max_sheet_id {
            s.push_str(&format!(r#"<sheetId val="{}"/>"#, i));
        }
        s.push_str("</sheetIdMap>");
        s.push_str("</header>");
    }

    s
}

pub(super) fn revision_log_xml(revision_log: &RevisionLog) -> String {
    todo!()
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
