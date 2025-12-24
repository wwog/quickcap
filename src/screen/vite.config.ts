import { defineConfig, Plugin } from "vite";
import { viteSingleFile } from "vite-plugin-singlefile"

function htmlMinify(): Plugin {
  return {
    name: 'html-minify',
    transform(code, id) {
      if (id.endsWith('.html')) {
        return {
          code: code
            .replace(/<!--[\s\S]*?-->/g, '')
            .replace(/\s+/g, ' ')
            .replace(/>\s+</g, '><')
            .trim(),
          map: null
        };
      }
      return null;
    },
    async generateBundle(_options, bundle) {
      for (const [fileName, chunk] of Object.entries(bundle)) {
        if (fileName.endsWith('.html') && 'source' in chunk) {
          const originalSource = chunk.source as string;
          chunk.source = originalSource
            .replace(/<!--[\s\S]*?-->/g, '')
            .replace(/\s+/g, ' ')
            .replace(/>\s+</g, '><')
            .trim();
        }
      }
    }
  };
}

function removeConsole(): Plugin {
  return {
    name: 'remove-console',
    transform(code) {
      return {
        code: code
          .replace(/console\.(log|info|warn|error|debug)\([^)]*\);?/g, '{}')
          .replace(/debugger;?/g, '{}'),
        map: null
      };
    }
  };
}

export default defineConfig(({ command, mode }) => {
  console.log("🚀 ~ command, mode:", command, mode);
  const isProduction = mode === 'production';
  const plugins = [viteSingleFile()];
  if (isProduction) {
    plugins.push(htmlMinify(), removeConsole());
  }

  return {
    base: "./",
    build: {
      assetsDir: "./assets",
      cssCodeSplit: false,
      minify: isProduction ? 'oxc' : false,
      cssMinify: isProduction,
      rollupOptions: {
        output: {
          manualChunks: undefined,
          inlineDynamicImports: true,
        }
      },
      outDir: "../app",
      sourcemap: false,
    },
    plugins,
  };
});
