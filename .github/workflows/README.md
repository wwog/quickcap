# GitHub Actions 工作流说明

## 标签推送未触发工作流的常见原因

### 1. 标签格式检查
确保标签以 `v` 开头，例如：
- ✅ 正确：`v0.1.0`, `v1.2.3`
- ❌ 错误：`0.1.0`, `version-0.1.0`

### 2. 推送方式检查
使用以下命令之一推送标签：

```bash
# 方式1：推送单个标签
git push origin v0.1.0

# 方式2：推送所有标签
git push origin --tags

# 方式3：推送标签并触发（推荐）
git tag v0.1.0
git push origin v0.1.0
```

### 3. 工作流文件检查
- 确保 `.github/workflows/release.yml` 已提交到仓库
- 确保文件在 `release` 分支或 `main/master` 分支上（取决于你的默认分支）

### 4. GitHub Actions 启用检查
- 进入仓库 Settings → Actions → General
- 确保 "Allow all actions and reusable workflows" 已启用

### 5. 检查工作流运行历史
- 进入仓库的 Actions 标签页
- 查看是否有失败的工作流运行
- 检查是否有权限问题

### 6. 标签推送验证
推送标签后，检查：
```bash
# 查看远程标签
git ls-remote --tags origin

# 确认标签已推送
git fetch --tags
git tag -l
```

### 7. 手动触发测试
如果标签推送不工作，可以尝试：
- 在 GitHub 网页上手动创建标签
- 或者先推送到分支，再创建标签

## 调试步骤

1. **检查工作流文件语法**：
   ```bash
   # 使用 GitHub Actions 的语法检查工具
   # 或在 GitHub 网页上查看工作流文件是否有错误提示
   ```

2. **查看工作流日志**：
   - 进入 Actions 标签页
   - 查看是否有任何错误信息

3. **测试触发**：
   ```bash
   # 创建测试标签
   git tag v0.0.1-test
   git push origin v0.0.1-test
   ```

## 常见问题

### Q: 标签推送了但没有触发工作流？
A: 检查：
- 标签格式是否为 `v*`
- 工作流文件是否在正确的分支上
- GitHub Actions 是否已启用

### Q: 工作流触发了但构建失败？
A: 查看 Actions 标签页中的错误日志，通常是：
- 依赖安装失败
- 构建脚本错误
- 权限问题

### Q: 如何查看工作流是否被触发？
A: 进入仓库的 Actions 标签页，所有工作流运行都会显示在那里。

