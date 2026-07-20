//! 定义首页悬浮二维码的展示内容与未来受控同源图片地址契约。

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HomeQrWidget {
    pub code: &'static str,
    pub title: &'static str,
    pub pending_label: &'static str,
    pub ready_label: &'static str,
    pub note: &'static str,
    pub image_alt: &'static str,
    pub open_label: &'static str,
    pub close_label: &'static str,
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
    pub const fn pending(labels: HomeQrLabels) -> Self {
        Self {
            code: labels.code,
            title: labels.title,
            pending_label: labels.pending,
            ready_label: labels.ready,
            note: labels.note,
            image_alt: labels.image_alt,
            open_label: labels.open,
            close_label: labels.close,
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
