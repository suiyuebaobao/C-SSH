//! 跨端请求摘要：对象键递归排序、数组顺序保留并编码为紧凑JSON。

use cloud_domain::{AppError, AppResult};
use serde::Serialize;
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

pub(crate) fn canonical<T: Serialize>(value: &T) -> AppResult<[u8; 32]> {
    let value = serde_json::to_value(value)
        .map_err(|_| AppError::Internal("canonical请求序列化失败".to_owned()))?;
    let mut encoded = serde_json::to_vec(&canonicalize(value))
        .map_err(|_| AppError::Internal("canonical请求摘要失败".to_owned()))?;
    let digest = Sha256::digest(&encoded).into();
    encoded.fill(0);
    Ok(digest)
}

fn canonicalize(value: Value) -> Value {
    match value {
        Value::Object(values) => {
            let mut entries = values.into_iter().collect::<Vec<_>>();
            entries.sort_unstable_by(|left, right| left.0.cmp(&right.0));
            let mut output = Map::new();
            for (key, value) in entries {
                output.insert(key, canonicalize(value));
            }
            Value::Object(output)
        }
        Value::Array(values) => Value::Array(values.into_iter().map(canonicalize).collect()),
        value => value,
    }
}

#[cfg(test)]
mod tests {
    use sha2::{Digest, Sha256};

    use super::{canonical, canonicalize};
    use serde_json::json;

    #[test]
    fn canonical_json_sorts_objects_and_preserves_arrays() {
        let value = json!({"z":{"b":2,"a":1},"a":[{"d":4,"c":3},2]});
        let encoded = serde_json::to_string(&canonicalize(value.clone())).unwrap();
        assert_eq!(encoded, r#"{"a":[{"c":3,"d":4},2],"z":{"a":1,"b":2}}"#);
        let expected: [u8; 32] = Sha256::digest(encoded).into();
        assert_eq!(canonical(&value).unwrap(), expected);
    }
}
