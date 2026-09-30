//! 验证外部下载 URL 默认不接收任何可能承载凭据的查询数据。

use cloud_domain::AppError;

use crate::validation;

#[test]
fn rejects_all_query_keys_including_disguised_credentials() {
    let query_strings = [
        "key=placeholder",
        "access_token=placeholder",
        "download_token=placeholder",
        "auth_code=placeholder",
        "ticket=placeholder",
        "version=0.6.16&channel=stable",
        "tokenizer_version=1&keynote=on",
    ];

    for query in query_strings {
        let url = format!("https://example.com/releases/setup.exe?{query}");
        let error = validation::external_url(&url).expect_err("外链查询参数必须被拒绝");
        assert!(
            matches!(error, AppError::Validation(message) if message == "外部来源 URL 不得包含查询参数"),
            "未按安全边界拒绝查询参数 {query}"
        );
    }
}

#[test]
fn accepts_a_clean_https_download_url() {
    assert!(validation::external_url("https://example.com/releases/setup.exe").is_ok());
}
