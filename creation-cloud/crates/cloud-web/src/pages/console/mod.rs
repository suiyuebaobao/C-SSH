//! 按用户控制台页面与动作拆分真实 SSR/HTMX 处理器。

mod common;
mod devices;
mod downloads;
mod hosts;
mod models;
mod overview;
mod profile;

pub(crate) use devices::{
    page as devices, rename::handle as rename_device, revoke::handle as revoke_device,
};
pub(crate) use downloads::page as downloads;
pub(crate) use hosts::{allowlist::handle as update_host_allowlist, page as hosts};
pub(crate) use models::page as models;
pub(crate) use overview::page as overview;
pub(crate) use profile::{
    change_password::handle as change_password, page as profile, update::handle as update_profile,
};
