//! 按用户控制台页面与动作拆分真实 SSR/HTMX 处理器。

mod common;
mod devices;
mod downloads;
mod models;
mod overview;
mod profile;
mod sync;
mod vault;

pub(crate) use devices::{
    page as devices, rename::handle as rename_device, revoke::handle as revoke_device,
};
pub(crate) use downloads::page as downloads;
pub(crate) use models::{
    create::handle as create_model, delete::handle as delete_model, page as models,
    update::handle as update_model,
};
pub(crate) use overview::page as overview;
pub(crate) use profile::{
    change_password::handle as change_password, page as profile, update::handle as update_profile,
};
pub(crate) use sync::page as sync;
pub(crate) use vault::page as vault;
