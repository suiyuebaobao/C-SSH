//! 定义首页悬浮二维码的展示内容与未来受控同源图片地址契约。

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct HomeQrWidget {
    pub code: String,
    pub title: String,
    pub pending_label: String,
    pub ready_label: String,
    pub note: String,
    pub image_alt: String,
    pub open_label: String,
    pub close_label: String,
    #[serde(skip)]
    image_src: Option<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HomeQrLabels {
    pub code: &'static str,
    pub title: &'static str,
    pub pending: &'static str,
    pub ready: &'static str,
    pub note: &'static str,
    pub image_alt: &'static str,
    pub open: &'static str,
    pub close: &'static str,
}

impl HomeQrWidget {
    #[must_use]
    pub fn pending(labels: HomeQrLabels) -> Self {
        Self {
            code: labels.code.to_owned(),
            title: labels.title.to_owned(),
            pending_label: labels.pending.to_owned(),
            ready_label: labels.ready.to_owned(),
            note: labels.note.to_owned(),
            image_alt: labels.image_alt.to_owned(),
            open_label: labels.open.to_owned(),
            close_label: labels.close.to_owned(),
            image_src: None,
        }
    }

    #[must_use]
    pub const fn has_image(&self) -> bool {
        self.image_src.is_some()
    }

    #[must_use]
    pub fn image_src(&self) -> Option<&str> {
        self.image_src.as_deref()
    }
}
