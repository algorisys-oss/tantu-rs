//! Native dialogs and notifications as plain requests (ADR 0017). Spec:
//! `docs/specs/platform/native-dialogs.md`.

use std::path::PathBuf;

/// What kind of file dialog.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileDialogKind {
    /// Choose one existing file.
    Open,
    /// Choose one or more existing files.
    OpenMany,
    /// Choose where to save a file.
    Save,
    /// Choose a folder.
    PickFolder,
}

/// A file dialog request (PLATFORM-NATIVE-01).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileDialogRequest {
    /// What to choose.
    pub kind: FileDialogKind,
    /// The dialog's title.
    pub title: Option<String>,
    /// Where it starts.
    pub directory: Option<PathBuf>,
    /// The suggested file name (save dialogs).
    pub file_name: Option<String>,
    /// (name, extensions without dots), e.g. ("Images", ["png", "jpg"]).
    pub filters: Vec<(String, Vec<String>)>,
}

impl FileDialogRequest {
    /// A request of `kind` with nothing else set.
    pub fn new(kind: FileDialogKind) -> Self {
        FileDialogRequest {
            kind,
            title: None,
            directory: None,
            file_name: None,
            filters: Vec::new(),
        }
    }

    /// Sets the title.
    pub fn title(self, title: impl Into<String>) -> Self {
        FileDialogRequest {
            title: Some(title.into()),
            ..self
        }
    }

    /// Sets the starting directory.
    pub fn directory(self, directory: impl Into<PathBuf>) -> Self {
        FileDialogRequest {
            directory: Some(directory.into()),
            ..self
        }
    }

    /// Sets the suggested file name.
    pub fn file_name(self, name: impl Into<String>) -> Self {
        FileDialogRequest {
            file_name: Some(name.into()),
            ..self
        }
    }

    /// Adds a filter: a name and extensions without dots.
    pub fn filter(mut self, name: impl Into<String>, extensions: &[&str]) -> Self {
        self.filters.push((
            name.into(),
            extensions.iter().map(|e| (*e).to_owned()).collect(),
        ));
        self
    }
}

/// A message dialog's icon and tone.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MessageLevel {
    /// Information.
    Info,
    /// A warning.
    Warning,
    /// An error.
    Error,
}

/// A message dialog's buttons.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MessageButtons {
    /// Ok.
    Ok,
    /// Ok and Cancel.
    OkCancel,
    /// Yes and No.
    YesNo,
}

/// A message dialog request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MessageRequest {
    /// Icon and tone.
    pub level: MessageLevel,
    /// The title.
    pub title: String,
    /// The text.
    pub description: String,
    /// The buttons.
    pub buttons: MessageButtons,
}

/// Which button closed a message dialog.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MessageResult {
    /// Ok.
    Ok,
    /// Cancel (or the dialog was dismissed).
    Cancel,
    /// Yes.
    Yes,
    /// No.
    No,
}

/// An OS notification.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Notification {
    /// The title.
    pub title: String,
    /// The text.
    pub body: String,
}

/// What a platform's native integrations support (FACADE-DESK-02).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct NativeCapabilities {
    /// File and message dialogs.
    pub dialogs: bool,
    /// Notifications.
    pub notifications: bool,
}

/// Identifies one dialog request.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct DialogId(pub u64);
