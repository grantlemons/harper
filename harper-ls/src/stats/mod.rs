mod lint_record;
mod lint_summary;

use std::io::Write;

pub use lint_record::LintRecord;
pub use lint_summary::LintSummary;
use tokio::io;

pub struct Stats {
    /// A record of the lints the user has applied.
    lints_applied: Vec<LintRecord>,
}

impl Stats {
    pub fn new() -> Self {
        Self {
            lints_applied: Vec::new(),
        }
    }

    /// Count the number of each kind of lint applied.
    pub fn summarize_lints_applied(&self) -> LintSummary {
        let mut summary = LintSummary::new();

        for lint in &self.lints_applied {
            summary.inc(lint.kind);
        }

        summary
    }

    pub fn lint_applied(&mut self, record: LintRecord) {
        self.lints_applied.push(record);
    }

    pub fn write_csv(&self, w: &mut impl Write) -> io::Result<()> {
        let mut writer = csv::WriterBuilder::new().has_headers(false).from_writer(w);

        for record in &self.lints_applied {
            writer.serialize(record)?;
        }

        Ok(())
    }
}

impl Default for Stats {
    fn default() -> Self {
        Self::new()
    }
}
