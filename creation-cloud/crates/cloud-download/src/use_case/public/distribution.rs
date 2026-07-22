//! 用类型区分已签发下载与未签发的 Range 拒绝响应。

use axum::{body::Body, http::Response};

pub(super) enum DistributionResult {
    LocalStream(Response<Body>),
    ExternalRedirect(Response<Body>),
    RangeRejected(Response<Body>),
}
