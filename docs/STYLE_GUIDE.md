# Documentation Style Guide

This guide defines the standards and conventions for diesel-gaussdb documentation to ensure consistency, clarity, and maintainability across all documentation.

## 📝 General Principles

### 1. Clarity First
- Use clear, concise language
- Avoid jargon unless necessary (and define it when used)
- Write for your audience (beginners vs. experts)
- Use active voice when possible

### 2. Consistency
- Follow established patterns and terminology
- Use consistent formatting and structure
- Maintain uniform code style across examples
- Apply consistent naming conventions

### 3. Completeness
- Cover all major use cases
- Include error handling examples
- Provide working code samples
- Document edge cases and limitations

### 4. Accessibility
- Support both English and Chinese languages
- Use inclusive language
- Provide multiple learning paths
- Include visual aids where helpful

## 🌐 Language Guidelines

### English Documentation
- Use American English spelling
- Write in present tense
- Use second person ("you") for instructions
- Keep sentences concise (max 20-25 words)

### Chinese Documentation (中文文档)
- Use simplified Chinese characters
- Maintain consistent technical term translations
- Use appropriate measure words
- Follow Chinese punctuation rules

### Common Technical Terms

| English | 中文 | Notes |
|---------|------|-------|
| Connection | 连接 | Database connection |
| Query | 查询 | SQL query |
| Transaction | 事务 | Database transaction |
| Migration | 迁移 | Schema migration |
| Backend | 后端 | Database backend |
| Pool | 连接池 | Connection pool |
| Cache | 缓存 | Query cache |
| Batch | 批量 | Batch operations |

## 📋 Document Structure

### Standard Template

```markdown
# Document Title

Brief description of what this document covers.

## 📋 Prerequisites (if applicable)
- List required knowledge
- Required software/versions
- Environment setup

## 🚀 Main Content
### Section 1
Content with examples

### Section 2
More content

## 💡 Examples
Working code examples

## 📝 Notes/Tips
Important notes or tips

## 🔗 Related Links
Links to related documentation

---
Footer with encouraging message
```

### File Naming Conventions

- Use kebab-case for file names: `getting-started.md`
- Use descriptive names: `connection-pooling.md` not `pools.md`
- Include language prefix for translations: `README_zh.md`
- Group related files in directories

### Directory Structure

```
docs/
├── en/                 # English documentation
│   ├── api/           # API reference
│   ├── guides/        # User guides
│   ├── examples/      # Code examples
│   └── reference/     # Technical reference
├── zh/                # Chinese documentation
│   ├── api/           # API 参考
│   ├── guides/        # 用户指南
│   ├── examples/      # 代码示例
│   └── reference/     # 技术参考
└── assets/            # Shared images/diagrams
```

## 🎨 Formatting Standards

### Headers

Use descriptive headers with emoji for visual appeal:

```markdown
# 🚀 Main Title
## 📋 Section Title
### 🔧 Subsection Title
```

### Code Blocks

Always specify the language for syntax highlighting:

```markdown
```rust
// Rust code example
let connection = establish_connection()?;
```

```sql
-- SQL example
SELECT * FROM users WHERE active = true;
```

```bash
# Shell commands
cargo run --example basic-crud
```
```

### Lists

Use consistent list formatting:

```markdown
- **Bold item**: Description
- **Another item**: More description
  - Nested item
  - Another nested item

1. **Numbered item**: Step description
2. **Next step**: More details
```

### Tables

Use tables for structured information:

```markdown
| Parameter | Type | Description | Default |
|-----------|------|-------------|---------|
| `host` | String | Database host | localhost |
| `port` | u16 | Database port | 5432 |
```

### Emphasis

- Use **bold** for important terms and UI elements
- Use *italics* for emphasis
- Use `code` for inline code, file names, and technical terms
- Use > blockquotes for important notes

### Links

- Use descriptive link text: `[Getting Started Guide](getting-started.md)`
- Use relative links for internal documentation
- Include external link indicators where helpful

## 💻 Code Examples

### Standards

1. **Working Code**: All examples must compile and run
2. **Complete Examples**: Include necessary imports and setup
3. **Error Handling**: Show proper error handling patterns
4. **Comments**: Add explanatory comments in code
5. **Realistic**: Use realistic data and scenarios

### Example Template

```rust
// src/example.rs
use diesel::prelude::*;
use diesel_gaussdb::GaussDbConnection;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Establish database connection
    let database_url = std::env::var("DATABASE_URL")?;
    let mut conn = GaussDbConnection::establish(&database_url)?;
    
    // Your example code here
    let users = users::table
        .select(User::as_select())
        .load(&mut conn)?;
    
    println!("Found {} users", users.len());
    
    Ok(())
}
```

### Code Comments

- Use `//` for single-line comments in Rust
- Use `/* */` for multi-line comments when needed
- Add comments to explain complex logic
- Include TODO comments for incomplete examples

## 📊 Visual Elements

### Emojis

Use emojis consistently for visual appeal:

- 🚀 Getting started, launches, new features
- 📋 Prerequisites, requirements, checklists
- 🔧 Configuration, setup, tools
- 💡 Examples, tips, ideas
- 📝 Notes, documentation, writing
- ⚠️ Warnings, important notes
- ✅ Success, completed items
- ❌ Errors, failures, problems
- 🔍 Search, find, investigate
- 📊 Data, statistics, metrics
- 🌐 Network, web, global
- 🔒 Security, authentication
- ⚡ Performance, speed, optimization

### Status Indicators

Use consistent status indicators:

- ✅ **Completed/Working**
- ⚠️ **Warning/Caution**
- ❌ **Error/Not Working**
- 🚧 **Work in Progress**
- 📋 **Todo/Planned**

## 🔍 Review Checklist

Before publishing documentation, verify:

### Content
- [ ] Information is accurate and up-to-date
- [ ] All code examples work correctly
- [ ] Links are valid and point to correct locations
- [ ] Grammar and spelling are correct
- [ ] Technical terms are used consistently

### Structure
- [ ] Follows the standard template
- [ ] Headers are properly nested
- [ ] Content is logically organized
- [ ] Navigation is clear

### Formatting
- [ ] Code blocks have language specified
- [ ] Tables are properly formatted
- [ ] Lists use consistent formatting
- [ ] Emphasis is used appropriately

### Accessibility
- [ ] Language is clear and inclusive
- [ ] Examples are realistic and helpful
- [ ] Multiple skill levels are considered
- [ ] Visual elements enhance understanding

## 🔄 Maintenance

### Regular Updates
- Review documentation with each release
- Update version numbers and compatibility information
- Refresh examples with current best practices
- Fix broken links and outdated information

### Community Feedback
- Monitor issues and discussions for documentation feedback
- Incorporate user suggestions and improvements
- Address common questions in documentation
- Update based on support forum discussions

### Version Control
- Use meaningful commit messages for documentation changes
- Tag documentation versions with releases
- Maintain changelog for major documentation updates
- Archive old versions when necessary

## 📞 Questions and Feedback

If you have questions about this style guide or suggestions for improvement:

1. Open an issue with the `documentation` label
2. Start a discussion in the community forum
3. Contact the documentation team directly

---

**Following these guidelines helps us maintain high-quality, consistent documentation that serves our community well!** 📚
