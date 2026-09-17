use crate::write::xml::{NS_PKG_REL, REL_REVISION_LOG, XML_DECL};
use crate::RevisionLog;

pub(super) fn revision_headers_xml(revision_logs: &[RevisionLog]) -> String {
    todo!()
}

pub(super) fn revision_log_xml(revision_log: &RevisionLog) -> String {
    todo!()
}

pub(super) fn revisions_user_names_xml(revision_logs: &[RevisionLog]) -> String {
    todo!()
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
