//! Global bilingual announcements with separate anonymous and administrator APIs.

mod model;
mod repository;
mod router;
mod service;
mod validation;

pub use model::{
    Announcement, AnnouncementLocale, AnnouncementStatus, CreateAnnouncementInput,
    CurrentAnnouncementResponse, PublicAnnouncement, ReplaceAnnouncementInput,
    TransitionAnnouncementInput,
};
pub use router::{management_router, public_router};
pub use service::Service;

#[cfg(test)]
mod tests;
