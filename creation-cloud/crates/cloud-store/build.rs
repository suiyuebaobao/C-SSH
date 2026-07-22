//! 让 Cargo 在数据库迁移目录变化时重新编译嵌入式迁移清单。
//! 迁移由 `sqlx::migrate!` 编入服务端，新增文件不得继续沿用旧二进制。

fn main() {
    println!("cargo:rerun-if-changed=../../migrations");
}
