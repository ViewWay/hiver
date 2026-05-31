# hiver-multipart

[![Crates.io](https://img.shields.io/crates/v/hiver-multipart)](https://crates.io/hiver-multipart)
[![Documentation](https://docs.rs/hiver-multipart/badge.svg)](https://docs.rs/hiver-multipart)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](../../LICENSE)

> Multipart file upload support for Hiver Framework
>
> Hiver框架的Multipart文件上传支持

---

## 📋 Overview / 概述

`hiver-multipart` provides multipart/form-data file upload support, equivalent to Spring's `MultipartFile` and `@RequestPart`.

`hiver-multipart` 提供 multipart/form-data 文件上传支持，等价于Spring的`MultipartFile`和`@RequestPart`。

**Key Features** / **核心特性**:
- ✅ **Multipart** - Multipart form data handling
- ✅ **MultipartFile** - Individual file upload
- ✅ **Size Limits** - Configurable file size limits
- ✅ **Multiple Files** - Batch file upload
- ✅ **Field Extraction** - Extract form fields and files

---

## ✨ Features / 特性

| Feature | Spring Equivalent | Description | Status |
|---------|------------------|-------------|--------|
| **Multipart** | `MultipartHttpServletRequest` | Multipart form data | ✅ |
| **MultipartFile** | `MultipartFile` | File upload | ✅ |
| **@RequestPart** | `@RequestPart` | Part extractor | ✅ |

---

## 🚀 Quick Start / 快速开始

### Installation / 安装

```toml
[dependencies]
hiver-multipart = "0.1.0-alpha"
```

### Basic Usage / 基本用法

```rust
use hiver_multipart::{Multipart, MultipartResult};
use hiver_http::Request;

async fn upload_file(mut multipart: Multipart) -> MultipartResult<String> {
    while let Some(mut field) = multipart.next_field().await? {
        let name = field.name().to_string();
        let filename = field.filename().map(|s| s.to_string());
        let data = field.data();

        if let Some(fname) = filename {
            // Save file / 保存文件
            field.save_to(format!("/uploads/{}", fname)).await?;
            println!("Saved file: {} ({})", fname, data.len());
        } else {
            // Process form field / 处理表单字段
            println!("Field {}: {}", name, field.text()?);
        }
    }
    Ok("Upload successful".to_string())
}
```

---

## 📖 Multipart Handling / Multipart 处理

### Process All Fields / 处理所有字段

```rust
use hiver_multipart::Multipart;

async fn process_multipart(mut multipart: Multipart) -> MultipartResult<()> {
    while let Some(field) = multipart.next_field().await? {
        let name = field.name().to_string();

        if field.is_file() {
            let filename = field.filename().unwrap_or("unnamed").to_string();
            let size = field.size();

            println!("File upload: {} ({} bytes)", filename, size);

            // Save file / 保存文件
            field.save_to(format!("/uploads/{}", filename)).await?;
        } else {
            let value = field.text()?;
            println!("Field {}: {}", name, value);
        }
    }

    Ok(())
}
```

### Get Specific Field / 获取特定字段

```rust
use hiver_multipart::Multipart;

async fn get_file(mut multipart: Multipart) -> MultipartResult<Vec<u8>> {
    if let Some(field) = multipart.field("avatar").await? {
        Ok(field.data().to_vec())
    } else {
        Err(MultipartError::FieldNotFound("avatar".to_string()))
    }
}
```

### Get All Files / 获取所有文件

```rust
use hiver_multipart::Multipart;

async fn get_all_files(mut multipart: Multipart) -> MultipartResult<Vec<String>> {
    let files = multipart.files().await?;
    let mut filenames = Vec::new();

    for file in files {
        let filename = file.filename().unwrap_or("unnamed").to_string();
        file.save_to(format!("/uploads/{}", filename)).await?;
        filenames.push(filename);
    }

    Ok(filenames)
}
```

---

## 🎯 File Size Limits / 文件大小限制

```rust
use hiver_multipart::{Multipart, DEFAULT_MAX_FILE_SIZE};

// Use default limit (10MB) / 使用默认限制（10MB）
let multipart = Multipart::new(
    content_type,
    body,
    DEFAULT_MAX_FILE_SIZE,
)?;

// Custom limit / 自定义限制
let multipart = Multipart::new(
    content_type,
    body,
    50 * 1024 * 1024, // 50MB
)?;
```

---

## 🧪 Testing / 测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_multipart_field() {
        let field = MultipartField {
            name: "test".to_string(),
            filename: None,
            content_type: Some("text/plain".to_string()),
            data: Bytes::from("hello"),
        };

        assert_eq!(field.name(), "test");
        assert_eq!(field.text().unwrap(), "hello");
        assert!(!field.is_file());
    }

    #[tokio::test]
    async fn test_extract_boundary() {
        let ct = "multipart/form-data; boundary=----WebKitFormBoundary";
        let boundary = Multipart::extract_boundary(ct).unwrap();
        assert_eq!(boundary, "----WebKitFormBoundary");
    }
}
```

---

## 🚦 Roadmap / 路线图

### Phase 2: Core Multipart ✅ (Completed / 已完成)
- [x] Multipart form data handling
- [x] File upload support
- [x] Size limits
- [x] Field extraction

### Phase 3: Advanced Features 📋 (Planned / 计划中)
- [ ] Streaming file upload
- [ ] Progress tracking
- [ ] File validation
- [ ] Multiple file storage backends

---

## 📚 Documentation / 文档

- **API Documentation**: [docs.rs/hiver-multipart](https://docs.rs/hiver-multipart)
- **Examples**: [examples/upload_example.rs](../../examples/upload_example.rs)

---

## 🤝 Contributing / 贡献

We welcome contributions! Please see:

- [CONTRIBUTING.md](../../CONTRIBUTING.md)
- [Design Spec](../../docs/design-spec.md)
- [GitHub Issues](https://github.com/hiver-framework/hiver/issues)

---

## 📄 License / 许可证

Licensed under Apache License 2.0. See [LICENSE](../../LICENSE) for details.

---

## 🙏 Acknowledgments / 致谢

Hiver Multipart is inspired by:

- **[Spring Framework](https://spring.io/projects/spring-framework)** - `MultipartFile`, `@RequestPart`
- **[Multer](https://github.com/rousan/multer-rs)** - Multipart implementation
- **[Actix Multipart](https://docs.rs/actix-multipart/)** - File upload handling

---

**Built with ❤️ for file uploads**

**为文件上传构建 ❤️**
