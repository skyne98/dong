// ===================================================================================
// SERVICES ARCHITECTURE
// ===================================================================================
//
// Services provide app-global functionality as singletons. They are easily accessible
// and provide reactive state where appropriate. Services follow the pattern:
//
// - Thread-local singletons using thread_local!
// - Reactive state using Val<T> for UI integration
// - Clean APIs for functionality
//
// Usage:
//   use crate::services;
//   services::theme(|theme| { /* use theme */ });
//   services::toast_mut(|toast| { toast.success("Hello!"); });
//
// ===================================================================================

pub mod theme;
pub mod toast;
pub mod status;
pub mod session;

/// Access the theme service immutably
pub fn theme<F, R>(f: F) -> R
where
    F: FnOnce(&theme::ThemeService) -> R,
{
    theme::THEME_SERVICE.with(|s| f(&*s.borrow()))
}

/// Access the theme service mutably
pub fn theme_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut theme::ThemeService) -> R,
{
    theme::THEME_SERVICE.with(|s| f(&mut *s.borrow_mut()))
}

/// Access the toast service mutably
pub fn toast_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut toast::ToastService) -> R,
{
    toast::TOAST_SERVICE.with(|s| f(&mut *s.borrow_mut()))
}

/// Access the status service immutably
pub fn status<F, R>(f: F) -> R
where
    F: FnOnce(&status::StatusService) -> R,
{
    status::STATUS_SERVICE.with(|s| f(&*s.borrow()))
}

/// Access the status service mutably
pub fn status_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut status::StatusService) -> R,
{
    status::STATUS_SERVICE.with(|s| f(&mut *s.borrow_mut()))
}

/// Access the session service immutably
pub fn session<F, R>(f: F) -> R
where
    F: FnOnce(&session::SessionService) -> R,
{
    session::SESSION_SERVICE.with(|s| f(&*s.borrow()))
}

/// Access the session service mutably
pub fn session_mut<F, R>(f: F) -> R
where
    F: FnOnce(&mut session::SessionService) -> R,
{
    session::SESSION_SERVICE.with(|s| f(&mut *s.borrow_mut()))
}
