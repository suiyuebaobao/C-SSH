//! 定义模板可直接消费的导航、页面区块、表单与产品预览模型。

use crate::{DocumentationContent, HomePageContent, Locale, RepositoryLink, TutorialPageContent};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PageId {
    Home,
    Documentation,
    Tutorials,
    Security,
    Downloads,
    Changelog,
    Faq,
    Feedback,
    Login,
    Register,
    Console,
    Devices,
    Sync,
    Models,
    Vault,
    Admin,
    AdminUsers,
    AdminDevices,
    AdminReleases,
    AdminAssets,
    AdminSite,
    AdminAudit,
    AdminFeedback,
}

impl PageId {
    pub const ALL: [Self; 23] = [
        Self::Home,
        Self::Documentation,
        Self::Tutorials,
        Self::Security,
        Self::Downloads,
        Self::Changelog,
        Self::Faq,
        Self::Feedback,
        Self::Login,
        Self::Register,
        Self::Console,
        Self::Devices,
        Self::Sync,
        Self::Models,
        Self::Vault,
        Self::Admin,
        Self::AdminUsers,
        Self::AdminDevices,
        Self::AdminReleases,
        Self::AdminAssets,
        Self::AdminSite,
        Self::AdminAudit,
        Self::AdminFeedback,
    ];

    pub const INDEXABLE: [Self; 6] = [
        Self::Home,
        Self::Documentation,
        Self::Tutorials,
        Self::Security,
        Self::Faq,
        Self::Feedback,
    ];

    #[must_use]
    pub const fn path(self) -> &'static str {
        match self {
            Self::Home => "/",
            Self::Tutorials => "/tutorials",
            Self::Documentation => "/docs/getting-started",
            Self::Security => "/security",
            Self::Downloads => "/downloads",
            Self::Changelog => "/changelog",
            Self::Faq => "/faq",
            Self::Feedback => "/feedback",
            Self::Login => "/login",
            Self::Register => "/register",
            Self::Console => "/console",
            Self::Devices => "/console/devices",
            Self::Sync => "/console/sync",
            Self::Models => "/console/models",
            Self::Vault => "/console/vault",
            Self::Admin => "/admin",
            Self::AdminUsers => "/admin/users",
            Self::AdminDevices => "/admin/devices",
            Self::AdminReleases => "/admin/releases",
            Self::AdminAssets => "/admin/assets",
            Self::AdminSite => "/admin/site",
            Self::AdminAudit => "/admin/audit",
            Self::AdminFeedback => "/admin/feedback",
        }
    }

    #[must_use]
    pub const fn is_indexable(self) -> bool {
        matches!(
            self,
            Self::Home
                | Self::Documentation
                | Self::Tutorials
                | Self::Security
                | Self::Faq
                | Self::Feedback
        )
    }

    #[must_use]
    pub fn localized_path(self, locale: Locale) -> String {
        localized_href(self.path(), locale)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NavigationItem {
    pub label: &'static str,
    pub href: String,
    pub active: bool,
}

impl NavigationItem {
    pub(crate) fn new(
        label: &'static str,
        target: PageId,
        current: PageId,
        locale: Locale,
    ) -> Self {
        Self {
            label,
            href: localized_href(target.path(), locale),
            active: target == current,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Action {
    pub label: &'static str,
    pub href: String,
    pub class_name: &'static str,
}

impl Action {
    pub(crate) fn new(
        label: &'static str,
        href: &'static str,
        class_name: &'static str,
        locale: Locale,
    ) -> Self {
        Self {
            label,
            href: localized_href(href, locale),
            class_name,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Metric {
    pub value: &'static str,
    pub label: &'static str,
    pub detail: &'static str,
}

impl Metric {
    pub(crate) const fn new(
        value: &'static str,
        label: &'static str,
        detail: &'static str,
    ) -> Self {
        Self {
            value,
            label,
            detail,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContentItem {
    pub badge: &'static str,
    pub title: &'static str,
    pub body: &'static str,
    pub meta: &'static str,
}

impl ContentItem {
    pub(crate) const fn new(
        badge: &'static str,
        title: &'static str,
        body: &'static str,
        meta: &'static str,
    ) -> Self {
        Self {
            badge,
            title,
            body,
            meta,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContentSection {
    pub anchor: &'static str,
    pub title: &'static str,
    pub lead: &'static str,
    pub items: Vec<ContentItem>,
}

impl ContentSection {
    pub(crate) fn new(
        anchor: &'static str,
        title: &'static str,
        lead: &'static str,
        items: Vec<ContentItem>,
    ) -> Self {
        Self {
            anchor,
            title,
            lead,
            items,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FaqItem {
    pub question: &'static str,
    pub answer: &'static str,
}

impl FaqItem {
    pub(crate) const fn new(question: &'static str, answer: &'static str) -> Self {
        Self { question, answer }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FormField {
    pub name: &'static str,
    pub label: &'static str,
    pub kind: &'static str,
    pub autocomplete: &'static str,
    pub placeholder: &'static str,
    pub required: bool,
}

impl FormField {
    pub(crate) const fn new(
        name: &'static str,
        label: &'static str,
        kind: &'static str,
        autocomplete: &'static str,
        placeholder: &'static str,
    ) -> Self {
        Self {
            name,
            label,
            kind,
            autocomplete,
            placeholder,
            required: true,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PageContent {
    pub id: PageId,
    pub path: &'static str,
    pub meta_title: &'static str,
    pub meta_description: &'static str,
    pub eyebrow: &'static str,
    pub heading: &'static str,
    pub lead: &'static str,
    pub actions: Vec<Action>,
    pub metrics: Vec<Metric>,
    pub sections: Vec<ContentSection>,
    pub home_page: Option<HomePageContent>,
    pub documentation_page: Option<DocumentationContent>,
    pub tutorial_page: TutorialPageContent,
    pub faqs: Vec<FaqItem>,
    pub fields: Vec<FormField>,
    pub form_action: &'static str,
    pub submit_label: &'static str,
    pub form_note: &'static str,
    pub local_navigation: Vec<NavigationItem>,
}

impl PageContent {
    pub(crate) fn new(
        id: PageId,
        meta_title: &'static str,
        meta_description: &'static str,
        eyebrow: &'static str,
        heading: &'static str,
        lead: &'static str,
    ) -> Self {
        Self {
            id,
            path: id.path(),
            meta_title,
            meta_description,
            eyebrow,
            heading,
            lead,
            actions: Vec::new(),
            metrics: Vec::new(),
            sections: Vec::new(),
            home_page: None,
            documentation_page: None,
            tutorial_page: TutorialPageContent::empty(),
            faqs: Vec::new(),
            fields: Vec::new(),
            form_action: "",
            submit_label: "",
            form_note: "",
            local_navigation: Vec::new(),
        }
    }

    #[must_use]
    pub(crate) fn with_actions(mut self, actions: Vec<Action>) -> Self {
        self.actions = actions;
        self
    }

    #[must_use]
    pub(crate) fn with_metrics(mut self, metrics: Vec<Metric>) -> Self {
        self.metrics = metrics;
        self
    }

    #[must_use]
    pub(crate) fn with_sections(mut self, sections: Vec<ContentSection>) -> Self {
        self.sections = sections;
        self
    }

    #[must_use]
    pub(crate) fn with_home_page(mut self, home_page: HomePageContent) -> Self {
        self.home_page = Some(home_page);
        self
    }

    #[must_use]
    pub(crate) fn with_documentation_page(
        mut self,
        documentation_page: DocumentationContent,
    ) -> Self {
        self.documentation_page = Some(documentation_page);
        self
    }

    #[must_use]
    pub(crate) fn with_tutorial_page(mut self, tutorial_page: TutorialPageContent) -> Self {
        self.tutorial_page = tutorial_page;
        self
    }

    #[must_use]
    pub(crate) fn with_faqs(mut self, faqs: Vec<FaqItem>) -> Self {
        self.faqs = faqs;
        self
    }

    #[must_use]
    pub(crate) fn with_form(
        mut self,
        action: &'static str,
        submit_label: &'static str,
        note: &'static str,
        fields: Vec<FormField>,
    ) -> Self {
        self.form_action = action;
        self.submit_label = submit_label;
        self.form_note = note;
        self.fields = fields;
        self
    }

    #[must_use]
    pub(crate) fn with_local_navigation(mut self, navigation: Vec<NavigationItem>) -> Self {
        self.local_navigation = navigation;
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SiteShell {
    pub locale: Locale,
    pub html_lang: &'static str,
    pub brand: &'static str,
    pub brand_note: &'static str,
    pub home_href: String,
    pub skip_label: &'static str,
    pub menu_label: &'static str,
    pub navigation: Vec<NavigationItem>,
    pub console_link: NavigationItem,
    pub login_link: NavigationItem,
    pub language_label: &'static str,
    pub language_href: String,
    pub alternate_lang: &'static str,
    pub utility_navigation_label: &'static str,
    pub github_repository: RepositoryLink,
    pub footer_summary: &'static str,
    pub footer_motto: &'static str,
    pub footer_note: &'static str,
    pub footer_navigation: Vec<NavigationItem>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SiteView {
    pub shell: SiteShell,
    pub page: PageContent,
}

pub(crate) fn localized_href(path: &str, locale: Locale) -> String {
    if locale == Locale::En && path.starts_with('/') {
        if path.starts_with("/console") || path.starts_with("/admin") {
            let separator = if path.contains('?') { '&' } else { '?' };
            format!("{path}{separator}lang=en")
        } else if path == "/" {
            "/en".to_owned()
        } else {
            format!("/en{path}")
        }
    } else {
        path.to_owned()
    }
}
