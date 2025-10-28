/// Which pane currently has focus
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    /// Device list pane has focus
    List,
    /// Device detail pane has focus
    Detail,
}

impl Focus {
    /// Switch to the other pane
    pub fn toggle(self) -> Self {
        match self {
            Focus::List => Focus::Detail,
            Focus::Detail => Focus::List,
        }
    }
}
