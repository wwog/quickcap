import { defineConfig } from "vite";
import { viteSingleFile } from "vite-plugin-singlefile"

export default defineConfig({
  // 设置基础路径为相对路径，这样生成的资源引用将以 ./ 开头而不是 / 开头
  base: "./",
  // 其他配置选项可以在这里添加
  build: {
    // 确保构建输出中的资源路径也是相对的
    assetsDir: "./assets",
    // 生成单个HTML文件，内联CSS和JS
    cssCodeSplit: false,
    rollupOptions: {
      output: {
        // 将所有JS打包到一个文件中
        manualChunks: undefined,
        // 内联资源
        inlineDynamicImports: true,
      },
    },
    // 设置构建输出目录
    outDir: "dist",
    // 生成source map
    sourcemap: false,
  },
  // 插件配置
  plugins: [
    viteSingleFile(),
  ],
});
