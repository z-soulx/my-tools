# My Tools — AI 工具集项目

## 项目概述

这是一个 AI 驱动的工具集合项目。每个工具独立存放在自己的文件夹中，彼此解耦，可以使用不同的技术栈。

## 项目结构

```
my-tools/
├── .claude/
│   └── CLAUDE.md          # 项目指引（本文件）
├── <tool-name>/           # 每个工具一个文件夹
│   ├── README.md          # 工具说明、用法、示例
│   ├── ...                # 工具源码及资源
│   └── ...
└── README.md              # 项目总览
```

## 约定

### 新建工具

1. 在项目根目录下创建以工具名命名的文件夹（kebab-case，如 `json-formatter`）
2. 文件夹内必须包含 `README.md`，说明：
   - 工具用途
   - 使用方法
   - 依赖与安装
   - 示例
3. 如有依赖，提供对应的依赖管理文件（`requirements.txt` / `package.json` / `go.mod` 等）
4. 每个工具应可独立运行，不依赖其他工具文件夹

### 命名规范

- 文件夹名：`kebab-case`（如 `image-compressor`、`markdown-to-pdf`）
- 代码风格：遵循对应语言的主流规范

### 技术栈

- 不限语言和框架，按工具需求选择最合适的技术
- 常用：Python、TypeScript/Node.js、Go、Bash
- 优先选择轻量、易部署的方案

### 代码质量

- 工具应有清晰的错误处理和用户提示
- CLI 工具应支持 `--help` 参数
- 尽量编写简洁、可读的代码，避免过度工程化

## 开发流程

1. 明确工具需求和使用场景
2. 创建工具文件夹并初始化
3. 实现核心功能
4. 编写 README.md
5. 测试验证

## Git 规范

- commit message 使用中文或英文均可，简洁描述变更
- 格式：`<type>: <description>`
  - `feat:` 新工具 / 新功能
  - `fix:` 修复
  - `docs:` 文档更新
  - `refactor:` 重构
  - `chore:` 杂项
