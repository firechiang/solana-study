pub mod instruction;
pub mod processor;
pub mod state;

// 该文件是项目导出依赖
#[cfg(not(feature = "no-entrypoint"))]
mod entrypoint;