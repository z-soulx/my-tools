# SkyTrace 飞书多维表格配置教程

本教程指导你完成飞书多维表格的创建和配置，用于 SkyTrace 的远端控制功能（kill switch、功能开关、版本管理）。

---

## 目录

1. [创建多维表格](#1-创建多维表格)
2. [设计表格字段](#2-设计表格字段)
3. [填写初始数据](#3-填写初始数据)
4. [获取 app_token 和 table_id](#4-获取-app_token-和-table_id)
5. [配置飞书应用权限](#5-配置飞书应用权限)
6. [将应用添加为表格协作者](#6-将应用添加为表格协作者)
7. [验证 API 是否通畅](#7-验证-api-是否通畅)
8. [日常管理操作指南](#8-日常管理操作指南)

---

## 1. 创建多维表格

**多维表格可以在任何地方创建**——个人空间、知识库、云盘文件夹都可以。最简单的方式：

1. 打开飞书（网页版或客户端）
2. 点击左侧导航栏的 **"+"** 或顶部 **"新建"** 按钮
3. 选择 **"多维表格"**
4. 命名为 **"SkyTrace 控制台"**（名字随意，方便你找到即可）

> 建议放在**个人空间**中，确保只有你自己能编辑。如果放在知识库或共享文件夹中，其他人可能有编辑权限。

---

## 2. 设计表格字段

创建多维表格后，默认会有一个空数据表。你需要把默认的列删除/修改，设置以下字段：

| 字段名 | 字段类型 | 说明 |
|--------|---------|------|
| `enabled` | **复选框** | 总开关。取消勾选 = 所有客户端立即锁定 |
| `min_version` | **文本** | 最低允许版本号，如 `0.1.0`。低于此版本的客户端会被强制要求更新 |
| `latest_version` | **文本** | 当前最新版本号，如 `0.2.0`。高于客户端版本时弹出更新提示 |
| `message` | **文本** | 锁定时显示的说明文字，如 `该工具已停用，请联系管理员` |
| `update_url_mac` | **URL** | macOS 安装包的下载地址 |
| `update_url_win` | **URL** | Windows 安装包的下载地址 |
| `update_notes` | **文本** | 版本更新说明，如 `修复了XX问题，新增YY功能` |
| `features` | **文本** | JSON 格式的功能开关（见下方说明） |
| `announcement_text` | **文本** | 公告内容。留空则不显示公告 |
| `announcement_type` | **单选** | 公告级别。添加三个选项：`info`、`warning`、`error` |

### 如何设置字段类型

1. 点击列标题右侧的 **下拉箭头**
2. 选择 **"编辑字段"** 或 **"修改字段类型"**
3. 选择对应的类型（文本、复选框、URL、单选等）

### features 字段格式

这是一个 JSON 文本，用来控制各功能模块的开关：

```json
{"skynetQuery":true,"snapshotExport":true,"snapshotImport":true,"checklistEdit":true,"recoveryEdit":true,"trashAccess":true}
```

把某个值改为 `false` 即可禁用对应功能。

---

## 3. 填写初始数据

只需要 **一行记录**。点击表格底部的 **"+"** 新增一行，填入：

| 字段 | 初始值 |
|------|--------|
| enabled | **勾选** (true) |
| min_version | `0.1.0` |
| latest_version | `0.1.0` |
| message | `该工具已停用` |
| update_url_mac | （暂时留空，有安装包地址后填写） |
| update_url_win | （暂时留空） |
| update_notes | （暂时留空） |
| features | `{"skynetQuery":true,"snapshotExport":true,"snapshotImport":true,"checklistEdit":true,"recoveryEdit":true,"trashAccess":true}` |
| announcement_text | （留空 = 无公告） |
| announcement_type | （留空或选 `info`） |

---

## 4. 获取 app_token 和 table_id

这两个值是 SkyTrace 代码中调用飞书 API 的必要参数。

### 获取方式

在浏览器中打开你刚创建的多维表格，观察地址栏 URL：

```
https://xxx.feishu.cn/base/BascXXXXXXXXXXXX?table=tblYYYYYYYYYYYY&view=vewZZZZZZZZZZZZ
```

- **app_token** = `BascXXXXXXXXXXXX`（`/base/` 后面的部分，到 `?` 之前）
- **table_id** = `tblYYYYYYYYYYYY`（`table=` 后面的部分，到 `&` 之前）

### 记录下来

请把以下三组值记好，后续配置代码时会用到：

```
App ID:     cli_a956cd3423f95ccb      （已有）
App Secret: s5gvtl3xxeXWXalKwnz8WdUQHbo2un7b （已有）
app_token:  Basc________________       （从 URL 获取）
table_id:   tbl_________________       （从 URL 获取）
```

---

## 5. 配置飞书应用权限

你已经创建了飞书应用（App ID: `cli_a956cd3423f95ccb`），现在需要给它配置多维表格的读取权限。

### 步骤

1. 打开 [飞书开放平台](https://open.feishu.cn/app)，登录
2. 在 **"我的应用"** 中找到你创建的应用，点击进入
3. 左侧菜单找到 **"开发配置"** → **"权限管理"**
4. 点击 **"开通权限"** 按钮
5. 在搜索框中搜索以下权限并逐一开通：

| 权限名称 | 权限标识 | 用途 |
|---------|---------|------|
| 查看、评论和编辑多维表格 | `bitable:app` | 读取多维表格数据（如果搜不到，搜 `bitable`） |

> 如果只有 `bitable:app:readonly`（只读），选这个也可以。我们只需要读取权限。

6. 勾选后点击 **"确认开通"**

### 发布应用使权限生效

权限配置后需要发布才能生效：

1. 左侧菜单 → **"版本管理与发布"**
2. 点击 **"创建版本"**
3. 填写版本号（如 `1.0.0`）和更新说明（如 `初始版本`）
4. 可用性范围：选择你自己（或不限制）
5. 点击 **"保存"** → **"申请发布"**
6. 个人账号通常会自动审核通过；如果是企业账号，需要管理员审批

---

## 6. 将应用添加为表格协作者

**这一步非常关键！** 即使应用有 API 权限，如果没有被添加为表格的协作者，API 调用会返回 403 权限错误。

### 方法（推荐）

1. 在飞书中打开你创建的 **"SkyTrace 控制台"** 多维表格
2. 点击右上角的 **"..."**（更多按钮）
3. 在下拉菜单中找到 **"更多"** → **"添加文档应用"**
   - 如果看不到这个选项，尝试：右上角 **"分享"** 按钮 → 搜索你的应用名称
4. 搜索你创建的应用名称（如 "SkyTrace 控制台"）
5. 选择该应用，权限设为 **"可阅读"** 或 **"可编辑"**（可阅读即可）
6. 确认添加

### 找不到"添加文档应用"？

不同版本的飞书界面可能略有不同。备选方法：

- 点击多维表格右上角的 **"分享"** → 在邀请框中输入应用名称
- 或者在多维表格的 **"设置"** → **"权限设置"** 中添加

### 验证是否添加成功

添加后，在表格的协作者/分享列表中应该能看到你的应用名称。

---

## 7. 验证 API 是否通畅

完成上述所有步骤后，可以用以下方式验证 API 是否能正常工作。

### 方法一：飞书 API 调试台（推荐）

1. 打开 [飞书 API 调试台](https://open.feishu.cn/api-explorer/cli_a956cd3423f95ccb?apiName=list&from=op_doc&project=bitable&resource=app.table.record&version=v1)
2. 选择 **"多维表格"** → **"列出记录"**
3. 填入 `app_token` 和 `table_id`
4. 选择认证方式为 **tenant_access_token**
5. 点击 **"试一试"**
6. 如果返回了你填写的记录数据，说明配置成功

### 方法二：命令行测试

**Step 1: 获取 tenant_access_token**

```bash
curl -X POST 'https://open.feishu.cn/open-apis/auth/v3/tenant_access_token/internal/' \
  -H 'Content-Type: application/json' \
  -d '{
    "app_id": "cli_a956cd3423f95ccb",
    "app_secret": "s5gvtl3xxeXWXalKwnz8WdUQHbo2un7b"
  }'
```

成功返回示例：

```json
{
  "code": 0,
  "msg": "ok",
  "tenant_access_token": "t-g1xxxxxxxxxxxxxxxxxxxxxx",
  "expire": 7200
}
```

**Step 2: 读取 Bitable 记录**

用上一步获取的 token 替换 `{TOKEN}`，用你的值替换 `{APP_TOKEN}` 和 `{TABLE_ID}`：

```bash
curl -X GET 'https://open.feishu.cn/open-apis/bitable/v1/apps/{APP_TOKEN}/tables/{TABLE_ID}/records?page_size=1' \
  -H 'Authorization: Bearer {TOKEN}'
```

成功返回示例（简化）：

```json
{
  "code": 0,
  "data": {
    "items": [
      {
        "record_id": "recXXXXXX",
        "fields": {
          "enabled": true,
          "min_version": "0.1.0",
          "latest_version": "0.1.0",
          "message": "该工具已停用",
          "features": "{\"skynetQuery\":true, ...}"
        }
      }
    ]
  }
}
```

### 常见错误排查

| 错误码 | 含义 | 解决方案 |
|--------|------|---------|
| `99991663` | tenant_access_token 无效 | 重新获取 token（有效期 2 小时） |
| `91403` 或 `1063002` | 权限不足 | 确认已完成第 5、6 步：开通 API 权限 + 添加应用为协作者 |
| `1254043` | app_token 不存在 | 检查 URL 中复制的 app_token 是否正确 |
| `1254044` | table_id 不存在 | 检查 URL 中复制的 table_id 是否正确 |

---

## 8. 日常管理操作指南

配置完成后，日常管理非常简单——直接在飞书 App（手机/电脑）或网页中编辑多维表格即可。

### 禁用所有客户端（kill switch）

1. 打开 "SkyTrace 控制台" 多维表格
2. 取消勾选 `enabled` 复选框
3. 在 `message` 中填写提示信息（如 "系统维护中，请稍后再试"）
4. 保存（自动保存）
5. 下次任何客户端启动时都会看到锁定界面

### 恢复使用

1. 重新勾选 `enabled`
2. 客户端重启后即可正常使用

### 强制所有用户升级到新版本

1. 修改 `min_version` 为新版本号（如 `0.3.0`）
2. 填写 `update_url_mac` 和 `update_url_win` 为新安装包的下载地址
3. 低于 0.3.0 的客户端将显示"请更新"界面

### 发布新版本（非强制）

1. 修改 `latest_version` 为新版本号
2. 填写 `update_notes` 更新说明
3. 填写 `update_url_*` 下载地址
4. 客户端启动时会弹出"有新版本"提示，用户可选择稍后更新

### 关闭某个功能

1. 编辑 `features` 字段，将对应功能设为 `false`
2. 例如禁用快照导出：`{"skynetQuery":true,"snapshotExport":false,...}`

### 发布公告

1. 在 `announcement_text` 中填写公告内容
2. `announcement_type` 选择级别：
   - `info` — 蓝色提示条
   - `warning` — 黄色警告条
   - `error` — 红色错误条
3. 清空 `announcement_text` 即可取消公告

### 永久停用（离职时）

以下任一操作都能让所有客户端永久不可用：

- **最简单**: 取消勾选 `enabled`，填写 `message` 说明
- **更彻底**: 在飞书开放平台删除/停用该应用 → API 直接失效
- **最彻底**: 删除多维表格 → API 返回错误 → 客户端锁定

---

## 附录：字段类型在 API 返回中的格式

飞书 Bitable API 返回的字段值格式与字段类型有关，以下是本教程中用到的字段的返回格式：

| 字段类型 | 飞书 API 返回格式 | 示例 |
|---------|-----------------|------|
| 复选框 | `true` / `false` | `"enabled": true` |
| 文本 | 富文本数组 `[{"text":"xxx","type":"text"}]` 或纯字符串 | `"min_version": [{"text":"0.1.0","type":"text"}]` |
| URL | `{"link":"https://...","text":"显示文本"}` 或纯文本 | 视配置而定 |
| 单选 | 选项文本字符串 | `"announcement_type": "info"` |

> 注意：**文本字段**在 API 中返回的是富文本数组格式，不是纯字符串。Rust 代码中需要做相应解析。这个细节在实现代码时会处理好。
