/**
 * 声明所有模块依赖（注意：该文件首先创建）
 */
pub mod error;

pub mod state;

pub mod processor;
pub mod instruction;

pub mod tools;

/**
 * 该文件是项目导出依赖
 */
#[cfg(not(feature = "no-entrypoint"))]
pub mod entrypoint;