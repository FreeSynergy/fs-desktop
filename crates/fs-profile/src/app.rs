// Profile — user profile, avatar, SSH keys, linked OIDC accounts, and personal capabilities.
//
// Auth flow:
//   1. App starts → `AuthState::LoggedOut`
//   2. User fills in username + password → `LoginSubmit` → async `authenticate_pam`
//   3. On success → `AuthState::LoggedIn { username, session_token, groups }`
//   4. Profile view shown (full profile editor)
//   5. Logout → session token invalidated → `AuthState::LoggedOut`
//
// Kanidm URL is read from `FS_AUTH_URL` env var; if absent, login is skipped
// (developer mode / offline). In that case the profile editor is shown directly.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Convenience: return translation as owned `String` for use in iced widgets.
fn tr(key: &str) -> String {
    fs_i18n::t(key).to_string()
}

// ── AuthState ─────────────────────────────────────────────────────────────────

/// Current authentication state of the profile app.
#[derive(Debug, Clone)]
pub enum AuthState {
    /// No session — login form is shown.
    LoggedOut,
    /// Login request in flight.
    Authenticating,
    /// Valid session obtained.
    LoggedIn {
        username: String,
        session_token: String,
        groups: Vec<String>,
    },
    /// Last login attempt failed.
    LoginError(String),
}

#[cfg(feature = "iced")]
use fs_gui_engine_iced::iced::{
    self,
    widget::{button, column, container, row, scrollable, text, text_input, Space},
    Alignment, Element, Length, Task,
};

// ── PersonalCapability ────────────────────────────────────────────────────────

/// Static metadata for a capability variant.
pub struct CapabilityMeta {
    pub icon: &'static str,
    /// Short, untranslated kind name shown in UI badges.
    pub kind: &'static str,
}

/// A personal resource this user has connected to the system.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PersonalCapability {
    /// User has a Telegram account → can receive personal bot messages.
    MessengerAccount {
        platform: String,
        username: String,
        #[serde(default)]
        verified: bool,
    },
    /// User has personal tasks in a task service (Vikunja).
    TaskManager { service_id: String },
    /// User has a personal mail inbox.
    Mailbox { service_id: String, address: String },
    /// User has a personal LLM assistant configured.
    LlmAssistant { provider: String, model: String },
}

impl PersonalCapability {
    /// Static metadata (icon, kind label) — single source of truth per variant.
    #[must_use]
    pub fn meta(&self) -> CapabilityMeta {
        match self {
            Self::MessengerAccount { .. } => CapabilityMeta {
                icon: "💬",
                kind: "Messenger",
            },
            Self::TaskManager { .. } => CapabilityMeta {
                icon: "✅",
                kind: "Task Manager",
            },
            Self::Mailbox { .. } => CapabilityMeta {
                icon: "📬",
                kind: "Mailbox",
            },
            Self::LlmAssistant { .. } => CapabilityMeta {
                icon: "🤖",
                kind: "LLM Assistant",
            },
        }
    }

    #[must_use]
    pub fn icon(&self) -> &'static str {
        self.meta().icon
    }
    #[must_use]
    pub fn kind_label(&self) -> &'static str {
        self.meta().kind
    }

    #[must_use]
    pub fn label(&self) -> String {
        match self {
            Self::MessengerAccount {
                platform, username, ..
            } => {
                format!("{} (@{})", capitalize(platform), username)
            }
            Self::TaskManager { service_id } => format!("{} ({})", self.kind_label(), service_id),
            Self::Mailbox { address, .. } => format!("{} ({})", self.kind_label(), address),
            Self::LlmAssistant { provider, model } => format!("LLM: {provider} / {model}"),
        }
    }
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

// ── Structs ───────────────────────────────────────────────────────────────────

/// A linked OIDC identity from an external provider.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LinkedAccount {
    pub provider: String,
    pub subject: String,
    pub username: String,
}

/// User profile data.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct UserProfile {
    pub display_name: String,
    pub email: String,
    pub avatar_url: Option<String>,
    pub bio: String,
    pub ssh_keys: Vec<SshKey>,
    pub timezone: String,
    #[serde(default)]
    pub linked_accounts: Vec<LinkedAccount>,
    #[serde(default)]
    pub personal_capabilities: Vec<PersonalCapability>,
}

/// An SSH public key entry.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SshKey {
    pub label: String,
    pub public_key: String,
    pub added_at: String,
}

impl UserProfile {
    fn path() -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        PathBuf::from(home)
            .join(".config")
            .join("fsn")
            .join("profile.toml")
    }

    /// Load profile from `~/.config/fsn/profile.toml`. Returns default if absent.
    #[must_use]
    pub fn load() -> Self {
        let path = Self::path();
        let content = std::fs::read_to_string(&path).unwrap_or_default();
        toml::from_str(&content).unwrap_or_default()
    }

    /// Save profile to `~/.config/fsn/profile.toml`.
    ///
    /// # Errors
    /// Returns an error string if the directory cannot be created or the file cannot be written.
    pub fn save(&self) -> Result<(), String> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let content = toml::to_string_pretty(self).map_err(|e| e.to_string())?;
        std::fs::write(&path, content).map_err(|e| e.to_string())
    }
}

// ── ProfileMessage ─────────────────────────────────────────────────────────────

/// All messages for the profile application.
#[derive(Debug, Clone)]
pub enum ProfileMessage {
    // ── Auth / login ──────────────────────────────────────────────────────────
    LoginUsernameChanged(String),
    LoginPasswordChanged(String),
    LoginSubmit,
    /// Result of `authenticate_pam`: `Ok((username, token, groups))` | `Err(message)`
    LoginResult(Result<(String, String, Vec<String>), String>),
    Logout,
    /// Result of `invalidate_session` after logout (ignored on error)
    LogoutDone,

    // ── Form fields ───────────────────────────────────────────────────────────
    DisplayNameChanged(String),
    EmailChanged(String),
    BioChanged(String),
    TimezoneChanged(String),
    AvatarRemove,

    // ── SSH keys ──────────────────────────────────────────────────────────────
    ShowAddKey,
    HideAddKey,
    NewKeyLabelChanged(String),
    NewKeyValueChanged(String),
    AddKey,
    RemoveKey(usize),

    // ── Linked accounts ───────────────────────────────────────────────────────
    ShowLinkAccount,
    HideLinkAccount,
    LinkProviderChanged(String),
    LinkSubjectChanged(String),
    LinkUsernameChanged(String),
    LinkAdd,
    LinkRemove(usize),

    // ── Persistence ───────────────────────────────────────────────────────────
    Save,
    SaveResult(Result<(), String>),
}

// ── ProfileApp ────────────────────────────────────────────────────────────────

/// Profile application state (iced-based MVU).
#[derive(Debug)]
pub struct ProfileApp {
    // ── Auth state ────────────────────────────────────────────────────────────
    pub auth: AuthState,
    /// Kanidm base URL — read from `FS_AUTH_URL` env var. Empty = offline mode.
    pub kanidm_url: String,
    pub login_username: String,
    pub login_password: String,

    // ── Profile data ──────────────────────────────────────────────────────────
    pub profile: UserProfile,
    pub save_msg: Option<String>,
    pub save_error: bool,

    // ── SSH key form ─────────────────────────────────────────────────────────
    pub show_add_key: bool,
    pub new_key_label: String,
    pub new_key_value: String,

    // ── Linked account form ───────────────────────────────────────────────────
    pub show_link: bool,
    pub link_provider: String,
    pub link_subject: String,
    pub link_username: String,
}

impl Default for ProfileApp {
    fn default() -> Self {
        Self::new()
    }
}

impl ProfileApp {
    /// Create a new profile app, loading the profile from disk.
    ///
    /// If `FS_AUTH_URL` is set, the login screen is shown first.
    /// If it is absent, offline/developer mode is assumed and the profile
    /// editor is shown directly (no authentication required).
    #[must_use]
    pub fn new() -> Self {
        let kanidm_url = std::env::var("FS_AUTH_URL").unwrap_or_default();
        let auth = if kanidm_url.is_empty() {
            // Offline / developer mode — skip login.
            AuthState::LoggedOut
        } else {
            AuthState::LoggedOut
        };
        Self {
            auth,
            kanidm_url,
            login_username: String::new(),
            login_password: String::new(),
            profile: UserProfile::load(),
            save_msg: None,
            save_error: false,
            show_add_key: false,
            new_key_label: String::new(),
            new_key_value: String::new(),
            show_link: false,
            link_provider: String::new(),
            link_subject: String::new(),
            link_username: String::new(),
        }
    }

    /// Whether the app is in offline mode (no Kanidm URL configured).
    #[must_use]
    pub fn is_offline(&self) -> bool {
        self.kanidm_url.is_empty()
    }
}

// ── update() ─────────────────────────────────────────────────────────────────

#[cfg(feature = "iced")]
impl ProfileApp {
    pub fn update(&mut self, msg: ProfileMessage) -> Task<ProfileMessage> {
        match msg {
            // ── Auth / login ──────────────────────────────────────────────────
            ProfileMessage::LoginUsernameChanged(v) => self.login_username = v,
            ProfileMessage::LoginPasswordChanged(v) => self.login_password = v,
            ProfileMessage::LoginSubmit => {
                if self.login_username.is_empty() || self.login_password.is_empty() {
                    return Task::none();
                }
                self.auth = AuthState::Authenticating;
                let url = self.kanidm_url.clone();
                let username = self.login_username.clone();
                let password = self.login_password.clone();
                return Task::perform(
                    async move {
                        use fs_auth::backend::KanidmBackend;
                        use fs_auth::pam::PamProvider;
                        let backend = KanidmBackend::new(&url, "", "", "");
                        backend
                            .authenticate_pam(&username, &password)
                            .await
                            .map(|id| (id.username, id.session_token, id.groups))
                            .map_err(|e| e.to_string())
                    },
                    ProfileMessage::LoginResult,
                );
            }
            ProfileMessage::LoginResult(Ok((username, token, groups))) => {
                self.login_password.clear();
                self.auth = AuthState::LoggedIn {
                    username,
                    session_token: token,
                    groups,
                };
            }
            ProfileMessage::LoginResult(Err(msg)) => {
                self.auth = AuthState::LoginError(msg);
            }
            ProfileMessage::Logout => {
                let token = match &self.auth {
                    AuthState::LoggedIn { session_token, .. } => session_token.clone(),
                    _ => String::new(),
                };
                self.auth = AuthState::LoggedOut;
                self.login_username.clear();
                if token.is_empty() {
                    return Task::none();
                }
                let url = self.kanidm_url.clone();
                return Task::perform(
                    async move {
                        use fs_auth::backend::KanidmBackend;
                        use fs_auth::sso::SsoProvider;
                        let backend = KanidmBackend::new(&url, "", "", "");
                        let _ = backend.invalidate_session(&token).await;
                    },
                    |()| ProfileMessage::LogoutDone,
                );
            }
            ProfileMessage::LogoutDone => {}

            // ── Form fields ───────────────────────────────────────────────────
            ProfileMessage::DisplayNameChanged(v) => self.profile.display_name = v,
            ProfileMessage::EmailChanged(v) => self.profile.email = v,
            ProfileMessage::BioChanged(v) => self.profile.bio = v,
            ProfileMessage::TimezoneChanged(v) => self.profile.timezone = v,
            ProfileMessage::AvatarRemove => self.profile.avatar_url = None,

            ProfileMessage::ShowAddKey => self.show_add_key = true,
            ProfileMessage::HideAddKey => {
                self.show_add_key = false;
                self.new_key_label.clear();
                self.new_key_value.clear();
            }
            ProfileMessage::NewKeyLabelChanged(v) => self.new_key_label = v,
            ProfileMessage::NewKeyValueChanged(v) => self.new_key_value = v,
            ProfileMessage::AddKey => {
                if !self.new_key_label.is_empty() && !self.new_key_value.is_empty() {
                    self.profile.ssh_keys.push(SshKey {
                        label: self.new_key_label.clone(),
                        public_key: self.new_key_value.clone(),
                        added_at: chrono::Local::now().format("%Y-%m-%d").to_string(),
                    });
                    self.new_key_label.clear();
                    self.new_key_value.clear();
                    self.show_add_key = false;
                }
            }
            ProfileMessage::RemoveKey(idx) => {
                if idx < self.profile.ssh_keys.len() {
                    self.profile.ssh_keys.remove(idx);
                }
            }

            ProfileMessage::ShowLinkAccount => self.show_link = true,
            ProfileMessage::HideLinkAccount => {
                self.show_link = false;
                self.link_provider.clear();
                self.link_subject.clear();
                self.link_username.clear();
            }
            ProfileMessage::LinkProviderChanged(v) => self.link_provider = v,
            ProfileMessage::LinkSubjectChanged(v) => self.link_subject = v,
            ProfileMessage::LinkUsernameChanged(v) => self.link_username = v,
            ProfileMessage::LinkAdd => {
                if !self.link_provider.is_empty() && !self.link_subject.is_empty() {
                    self.profile.linked_accounts.push(LinkedAccount {
                        provider: self.link_provider.clone(),
                        subject: self.link_subject.clone(),
                        username: self.link_username.clone(),
                    });
                    self.show_link = false;
                    self.link_provider.clear();
                    self.link_subject.clear();
                    self.link_username.clear();
                }
            }
            ProfileMessage::LinkRemove(idx) => {
                if idx < self.profile.linked_accounts.len() {
                    self.profile.linked_accounts.remove(idx);
                }
            }

            ProfileMessage::Save => {
                let profile = self.profile.clone();
                return Task::perform(async move { profile.save() }, ProfileMessage::SaveResult);
            }
            ProfileMessage::SaveResult(result) => match result {
                Ok(()) => {
                    self.save_msg = Some(fs_i18n::t("profile.saved").into());
                    self.save_error = false;
                }
                Err(e) => {
                    self.save_msg = Some(e);
                    self.save_error = true;
                }
            },
        }
        Task::none()
    }
}

// ── view() ───────────────────────────────────────────────────────────────────

#[cfg(feature = "iced")]
impl ProfileApp {
    #[must_use]
    pub fn view(&self) -> Element<'_, ProfileMessage> {
        // Show login screen unless offline or already logged in.
        if !self.is_offline() {
            match &self.auth {
                AuthState::LoggedOut | AuthState::LoginError(_) | AuthState::Authenticating => {
                    return self.view_login();
                }
                AuthState::LoggedIn { .. } => {}
            }
        }
        self.view_profile()
    }

    fn view_login(&self) -> Element<'_, ProfileMessage> {
        let title = text(tr("auth-title")).size(24);

        let username_ph = tr("auth-username-placeholder");
        let username_input = text_input(&username_ph, &self.login_username)
            .on_input(ProfileMessage::LoginUsernameChanged)
            .padding([8, 12])
            .size(14)
            .width(Length::Fill);

        let password_ph = tr("auth-password-placeholder");
        let password_input = text_input(&password_ph, &self.login_password)
            .on_input(ProfileMessage::LoginPasswordChanged)
            .secure(true)
            .padding([8, 12])
            .size(14)
            .width(Length::Fill);

        let login_btn = button(text(tr("auth-btn-login")).size(14))
            .on_press(ProfileMessage::LoginSubmit)
            .padding([8, 24]);

        let status: Element<'_, ProfileMessage> = match &self.auth {
            AuthState::Authenticating => text(tr("auth-status-checking"))
                .size(13)
                .color(iced::Color::from_rgb(0.5, 0.5, 0.7))
                .into(),
            AuthState::LoginError(msg) => text(msg)
                .size(13)
                .color(iced::Color::from_rgb(0.87, 0.27, 0.27))
                .into(),
            _ => Space::with_height(0).into(),
        };

        let content = column![
            title,
            Space::with_height(32),
            username_input,
            Space::with_height(12),
            password_input,
            Space::with_height(16),
            row![login_btn, Space::with_width(16), status]
                .align_y(Alignment::Center)
                .spacing(0),
        ]
        .spacing(0)
        .padding([48, 40])
        .max_width(400);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    }

    fn view_profile(&self) -> Element<'_, ProfileMessage> {
        // ── Header: title + logged-in user + logout button ────────────────────
        let title = text(tr("profile.title")).size(24);

        let header_row: Element<'_, ProfileMessage> =
            if let AuthState::LoggedIn { username, .. } = &self.auth {
                let user_label = text(username.as_str())
                    .size(13)
                    .color(iced::Color::from_rgb(0.5, 0.5, 0.6));
                let logout_btn = button(text(tr("auth-btn-logout")).size(12))
                    .on_press(ProfileMessage::Logout)
                    .padding([4, 10]);
                row![
                    title,
                    Space::with_width(Length::Fill),
                    user_label,
                    Space::with_width(8),
                    logout_btn,
                ]
                .align_y(Alignment::Center)
                .into()
            } else {
                title.into()
            };

        // ── Avatar row ────────────────────────────────────────────────────────
        let avatar_icon = text("👤").size(40);
        let remove_avatar_btn = button(text(tr("actions.remove")).size(12))
            .on_press(ProfileMessage::AvatarRemove)
            .padding([4, 10]);
        let avatar_row = row![avatar_icon, Space::with_width(16), remove_avatar_btn]
            .align_y(Alignment::Center)
            .spacing(8);

        // ── Display name + email ──────────────────────────────────────────────
        let name_placeholder = tr("profile.label.display_name");
        let name_label = text(name_placeholder.clone()).size(12);
        let name_input = text_input(&name_placeholder, &self.profile.display_name)
            .on_input(ProfileMessage::DisplayNameChanged)
            .padding([8, 12])
            .size(14)
            .width(Length::Fill);

        let email_placeholder = tr("profile.label.email");
        let email_label = text(email_placeholder.clone()).size(12);
        let email_input = text_input(&email_placeholder, &self.profile.email)
            .on_input(ProfileMessage::EmailChanged)
            .padding([8, 12])
            .size(14)
            .width(Length::Fill);

        let name_col = column![name_label, Space::with_height(4), name_input].spacing(0);
        let email_col = column![email_label, Space::with_height(4), email_input].spacing(0);
        let name_email_row = row![name_col, Space::with_width(12), email_col].spacing(0);

        // ── Bio ───────────────────────────────────────────────────────────────
        let bio_label = text(tr("profile.label.bio")).size(12);
        let bio_input = text_input("A short description...", &self.profile.bio)
            .on_input(ProfileMessage::BioChanged)
            .padding([8, 12])
            .size(14)
            .width(Length::Fill);

        // ── SSH Keys ─────────────────────────────────────────────────────────
        let ssh_title = text(tr("profile.section.ssh_keys")).size(16);
        let add_key_btn = button(text(tr("profile.btn.add_key")).size(13))
            .on_press(ProfileMessage::ShowAddKey)
            .padding([6, 14]);

        let mut ssh_items: Vec<Element<'_, ProfileMessage>> = self
            .profile
            .ssh_keys
            .iter()
            .enumerate()
            .map(|(idx, key)| {
                let remove_btn = button(text("✕").size(11))
                    .on_press(ProfileMessage::RemoveKey(idx))
                    .padding([2, 6]);
                row![
                    column![
                        text(&key.label).size(13),
                        text(key.public_key.chars().take(48).collect::<String>() + "…")
                            .size(10)
                            .color(iced::Color::from_rgb(0.5, 0.5, 0.6)),
                    ]
                    .spacing(2)
                    .width(Length::Fill),
                    remove_btn,
                ]
                .align_y(Alignment::Center)
                .spacing(8)
                .into()
            })
            .collect();

        if self.show_add_key {
            let key_label_input = text_input("Key label", &self.new_key_label)
                .on_input(ProfileMessage::NewKeyLabelChanged)
                .padding([6, 10])
                .size(13)
                .width(Length::Fill);
            let key_value_input = text_input("ssh-ed25519 AAAA...", &self.new_key_value)
                .on_input(ProfileMessage::NewKeyValueChanged)
                .padding([6, 10])
                .size(13)
                .width(Length::Fill);
            let confirm_btn = button(text(tr("actions.add")).size(13))
                .on_press(ProfileMessage::AddKey)
                .padding([6, 14]);
            let cancel_btn = button(text(tr("actions.cancel")).size(13))
                .on_press(ProfileMessage::HideAddKey)
                .padding([6, 14]);
            ssh_items.push(
                column![
                    key_label_input,
                    Space::with_height(4),
                    key_value_input,
                    Space::with_height(8),
                    row![confirm_btn, Space::with_width(8), cancel_btn].spacing(0),
                ]
                .spacing(0)
                .into(),
            );
        }

        // ── Linked Accounts ───────────────────────────────────────────────────
        let accounts_title = text(tr("profile.section.linked_accounts")).size(16);
        let add_account_btn = button(text(tr("profile.btn.link_account")).size(13))
            .on_press(ProfileMessage::ShowLinkAccount)
            .padding([6, 14]);

        let account_items: Vec<Element<'_, ProfileMessage>> = self
            .profile
            .linked_accounts
            .iter()
            .enumerate()
            .map(|(idx, acc)| {
                let remove_btn = button(text("✕").size(11))
                    .on_press(ProfileMessage::LinkRemove(idx))
                    .padding([2, 6]);
                row![
                    column![
                        text(&acc.provider).size(13),
                        text(&acc.username)
                            .size(11)
                            .color(iced::Color::from_rgb(0.5, 0.5, 0.6)),
                    ]
                    .spacing(2)
                    .width(Length::Fill),
                    remove_btn,
                ]
                .align_y(Alignment::Center)
                .spacing(8)
                .into()
            })
            .collect();

        // ── Save button + status ──────────────────────────────────────────────
        let save_btn = button(text(tr("actions.save")).size(14))
            .on_press(ProfileMessage::Save)
            .padding([8, 24]);

        let status: Element<'_, ProfileMessage> = if let Some(msg) = &self.save_msg {
            let color = if self.save_error {
                iced::Color::from_rgb(0.87, 0.27, 0.27)
            } else {
                iced::Color::from_rgb(0.13, 0.76, 0.37)
            };
            text(msg).size(13).color(color).into()
        } else {
            Space::with_height(0).into()
        };

        let content = column![
            header_row,
            Space::with_height(24),
            avatar_row,
            Space::with_height(24),
            name_email_row,
            Space::with_height(16),
            bio_label,
            Space::with_height(4),
            bio_input,
            Space::with_height(32),
            ssh_title,
            Space::with_height(8),
            column(ssh_items).spacing(8),
            Space::with_height(8),
            add_key_btn,
            Space::with_height(32),
            accounts_title,
            Space::with_height(8),
            column(account_items).spacing(8),
            Space::with_height(8),
            add_account_btn,
            Space::with_height(32),
            row![save_btn, Space::with_width(16), status]
                .align_y(Alignment::Center)
                .spacing(0),
        ]
        .spacing(0)
        .padding([24, 32])
        .max_width(640);

        container(scrollable(content))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

// ── Non-iced stubs ────────────────────────────────────────────────────────────

#[cfg(not(feature = "iced"))]
impl ProfileApp {
    pub fn update(&mut self, _msg: ProfileMessage) {}
}
