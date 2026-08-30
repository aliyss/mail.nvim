pub mod compose;
pub mod get;
pub mod list;
pub mod manage;
pub mod pagination;
pub mod selection;
pub mod thread;

pub use compose::{
    EmailCreate, EmailForward, EmailReply, EmailReplyAll, EmailSaveAsDraft, EmailSend,
};
pub use manage::{
    EmailCopy, EmailDelete, EmailFlagAdd, EmailFlagClear, EmailFlagRemove, EmailMove,
    EmailToggleRead,
};
pub use pagination::email_list_page;
pub use selection::{email_clear_selection, email_toggle_selection};
