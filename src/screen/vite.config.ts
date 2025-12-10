import { defineConfig } from 'vite';

export default defineConfig({
  // 设置基础路径为相对路径，这样生成的资源引用将以 ./ 开头而不是 / 开头
  base: './',
  // 其他配置选项可以在这里添加
  build: {
    // 确保构建输出中的资源路径也是相对的
    assetsDir: './assets',
  }
});