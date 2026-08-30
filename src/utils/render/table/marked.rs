//! Selection marker column for email list tables.
//!
//! [`MarkedTable`] wraps any list [`RenderTable`] and prepends a `Sel` column
//! whose cells are `>` for the selected rows. Parsing is header-name driven,
//! so the extra column is transparent to [`Table::from_buffer`]: the id,
//! subject, ... columns keep their positions relative to their headers.

use std::collections::HashSet;

use crate::utils::render::table::render::{RenderTable, RowBuilder};

/// Something that has an id used to match the selection marker.
pub trait HasId {
    /// The item's id.
    fn id(&self) -> &str;
}

/// A [`RenderTable`] that prepends a `Sel` column marking the rows whose
/// item id is in `selected`.
#[derive(Debug, Clone)]
pub struct MarkedTable<T: RenderTable> {
    data: T,
    selected: HashSet<String>,
    /// The row that is loading and the character drawn in its `Sel` cell.
    spinner: Option<(usize, char)>,
}

impl<T> std::ops::Deref for MarkedTable<T>
where
    T: RenderTable,
{
    type Target = [T::Item];

    fn deref(&self) -> &Self::Target {
        self.data.deref()
    }
}

impl<T: RenderTable> MarkedTable<T> {
    #[must_use]
    pub fn new(data: T, selected: HashSet<String>) -> Self {
        Self {
            data,
            selected,
            spinner: None,
        }
    }

    /// Marks `row` as loading: its `Sel` cell shows `ch` until the load ends.
    #[must_use]
    pub fn with_spinner(mut self, row: usize, ch: char) -> Self {
        self.spinner = Some((row, ch));
        self
    }

    /// The number of rows of the wrapped table.
    #[must_use]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Whether the wrapped table has no rows.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

impl<T> RenderTable for MarkedTable<T>
where
    T: RenderTable,
    T::Item: HasId,
{
    type Item = T::Item;

    fn headers(&self) -> Vec<String> {
        let mut headers = vec!["Sel".to_string()];
        headers.extend(self.data.headers());
        headers
    }

    fn rows(&self) -> Vec<RowBuilder> {
        self.data
            .rows()
            .into_iter()
            .enumerate()
            .zip(self.data.iter())
            .map(|((row_index, row), item)| {
                let marker = if let Some((spinner_row, ch)) = self.spinner
                    && spinner_row == row_index
                {
                    ch.to_string()
                } else if self.selected.contains(item.id()) {
                    ">".to_string()
                } else {
                    String::new()
                };
                // The selection marker gets its own color; the spinner stays
                // unstyled. The wrapped styles shift one column to the right.
                let marker_style = if marker == ">" {
                    Some("MailTableSelected")
                } else {
                    None
                };
                let mut cells = vec![marker];
                cells.extend(row.cells);
                let mut styles = vec![marker_style];
                styles.extend(row.styles);
                RowBuilder { cells, styles }
            })
            .collect()
    }

    fn from_headers_and_rows(headers: Vec<String>, rows: Vec<RowBuilder>) -> Self {
        // Only reached when parsing a rendered marked table; the `Sel` column
        // is ignored by the underlying impl (it parses by header name).
        Self {
            data: T::from_headers_and_rows(headers, rows),
            selected: HashSet::new(),
            spinner: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Stub {
        id: String,
    }

    impl HasId for Stub {
        fn id(&self) -> &str {
            &self.id
        }
    }

    impl RenderTable for Vec<Stub> {
        type Item = Stub;

        fn headers(&self) -> Vec<String> {
            vec!["ID".to_string(), "Name".to_string()]
        }

        fn rows(&self) -> Vec<RowBuilder> {
            self.iter()
                .map(|stub| {
                    RowBuilder::new()
                        .with_cell(stub.id.clone())
                        .with_cell(stub.id.clone())
                })
                .collect()
        }

        fn from_headers_and_rows(headers: Vec<String>, rows: Vec<RowBuilder>) -> Self {
            let id_index = headers
                .iter()
                .position(|h| h == "ID")
                .expect("ID header should be present");
            rows.into_iter()
                .map(|row| Stub {
                    id: row.cells.get(id_index).cloned().unwrap_or_default(),
                })
                .collect()
        }
    }

    #[test]
    fn headers_prepend_sel_column() {
        let marked = MarkedTable::new(vec![Stub { id: "1".into() }], HashSet::new());
        assert_eq!(
            marked.headers(),
            vec!["Sel".to_string(), "ID".to_string(), "Name".to_string()]
        );
    }

    #[test]
    fn selected_rows_are_marked() {
        let marked = MarkedTable::new(
            vec![Stub { id: "1".into() }, Stub { id: "2".into() }],
            HashSet::from(["1".to_string()]),
        );

        let rows = marked.rows();
        assert_eq!(rows[0].cells[0], ">");
        assert_eq!(rows[0].cells[1], "1");
        assert_eq!(rows[1].cells[0], "");
        assert_eq!(rows[1].cells[1], "2");
    }

    #[test]
    fn selected_marker_is_styled_unselected_is_not() {
        let marked = MarkedTable::new(
            vec![Stub { id: "1".into() }, Stub { id: "2".into() }],
            HashSet::from(["1".to_string()]),
        );

        let rows = marked.rows();
        assert_eq!(rows[0].styles[0], Some("MailTableSelected"));
        assert_eq!(rows[1].styles[0], None);
        // The wrapped styles shift one column to the right.
        assert_eq!(rows[0].styles[1], None);
    }

    #[test]
    fn spinner_overrides_the_marker_of_its_row() {
        let marked = MarkedTable::new(
            vec![Stub { id: "1".into() }, Stub { id: "2".into() }],
            HashSet::from(["1".to_string()]),
        )
        .with_spinner(1, '⠼');

        let rows = marked.rows();
        // The selected row keeps its marker, the loading row shows the
        // spinner instead of its (empty) marker.
        assert_eq!(rows[0].cells[0], ">");
        assert_eq!(rows[1].cells[0], "⠼");
    }

    #[test]
    fn parsing_ignores_the_sel_column() {
        let marked = MarkedTable::new(
            vec![Stub { id: "1".into() }, Stub { id: "2".into() }],
            HashSet::from(["1".to_string()]),
        );

        // Round-tripping through the header-driven parse keeps the ids.
        let parsed = Vec::<Stub>::from_headers_and_rows(marked.headers(), marked.rows());
        assert_eq!(
            parsed,
            vec![Stub { id: "1".into() }, Stub { id: "2".into() }]
        );
    }
}
