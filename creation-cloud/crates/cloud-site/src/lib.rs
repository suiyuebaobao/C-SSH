//! 提供 Creation-SSH 网站的纯内容模型与中英文内容服务。

mod content;
mod documentation;
mod home;
mod home_qr;
mod locale;
mod model;
mod repository;
mod service;
mod tutorial;

pub use documentation::{
    DocumentationContent, DocumentationGroup, DocumentationItem, DocumentationLink,
    DocumentationNotice, DocumentationPlatform, DocumentationScreenshot, DocumentationSection,
};
pub use home::{
    HomeFaqItem, HomeItem, HomeLayout, HomePageContent, HomePlatform, HomeSection, HomeTone,
};
pub use home_qr::{HomeQrLabels, HomeQrWidget};
pub use locale::Locale;
pub use model::{
    Action, ContentItem, ContentSection, FaqItem, FormField, Metric, NavigationItem, PageContent,
    PageId, SiteShell, SiteView,
};
pub use repository::RepositoryLink;
pub use service::{ContentService, content_service};
pub use tutorial::{Tutorial, TutorialPageContent, TutorialStep};
