//! 管理教学影片的元数据、受控上传、公开播放及管理员生命周期。

mod content;
mod model;
mod multipart;
mod repository;
mod router;
mod service;
mod storage;
mod validation;

pub use router::{management_router, public_router};
pub use service::Service;
