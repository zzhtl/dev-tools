use std::path::Path;
use std::process::Command;

/// 占位页哨兵：CI 用它断言嵌入的不是空壳前端。
const PLACEHOLDER_SENTINEL: &str = "<!--dev-tools-placeholder-->";

fn main() -> anyhow::Result<()> {
    println!("cargo:rerun-if-changed=web/src");
    println!("cargo:rerun-if-changed=web/public");
    println!("cargo:rerun-if-changed=web/package.json");
    println!("cargo:rerun-if-changed=web/index.html");
    println!("cargo:rerun-if-changed=web/vite.config.ts");
    println!("cargo:rerun-if-changed=web/tsconfig.json");
    println!("cargo:rerun-if-env-changed=DEV_TOOLS_SKIP_WEB_BUILD");

    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let web = root.join("web");
    let dist = root.join("dist");

    // 发布包/vendored 场景：没有 web/ 源码，只能用占位页。
    if !web.exists() {
        std::fs::create_dir_all(&dist)?;
        write_placeholder(&dist)?;
        println!("cargo:warning=web/ 不存在，跳过前端构建（使用占位页）");
        return Ok(());
    }

    // CI 场景：前端由外部只构建一次，多个 target 复用同一份 dist/。
    if skip_web_build() {
        let index = dist.join("index.html");
        anyhow::ensure!(
            index.exists(),
            "DEV_TOOLS_SKIP_WEB_BUILD 已设置，但 {} 不存在；请先在 web/ 下执行 bun run build",
            index.display()
        );
        let html = std::fs::read_to_string(&index)?;
        anyhow::ensure!(
            !html.contains(PLACEHOLDER_SENTINEL),
            "DEV_TOOLS_SKIP_WEB_BUILD 已设置，但 {} 是占位页而非真实前端产物",
            index.display()
        );
        println!("cargo:warning=DEV_TOOLS_SKIP_WEB_BUILD=1，复用已有 dist/");
        return Ok(());
    }

    // 默认路径：必须有 bun。这里刻意硬失败，避免静默产出只带占位页的空壳二进制。
    anyhow::ensure!(
        which("bun").is_some(),
        "未找到 bun，无法构建前端。请安装 bun（curl -fsSL https://bun.sh/install | bash），\
         或先自行构建 web/ 后使用 DEV_TOOLS_SKIP_WEB_BUILD=1 复用已有 dist/"
    );

    if !web.join("node_modules").exists() {
        run("bun", &["install"], &web)?;
    }
    run("bun", &["run", "build"], &web)?;

    let index = dist.join("index.html");
    anyhow::ensure!(
        index.exists(),
        "bun run build 结束后仍未生成 {}",
        index.display()
    );
    Ok(())
}

fn skip_web_build() -> bool {
    matches!(
        std::env::var("DEV_TOOLS_SKIP_WEB_BUILD").as_deref(),
        Ok("1") | Ok("true")
    )
}

fn which(cmd: &str) -> Option<std::path::PathBuf> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(format!("command -v {cmd}"))
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if path.is_empty() {
        None
    } else {
        Some(path.into())
    }
}

fn run(cmd: &str, args: &[&str], cwd: &Path) -> anyhow::Result<()> {
    let status = Command::new(cmd).args(args).current_dir(cwd).status()?;
    anyhow::ensure!(status.success(), "{cmd} {args:?} failed with {status}");
    Ok(())
}

fn write_placeholder(dist: &Path) -> anyhow::Result<()> {
    std::fs::write(
        dist.join("index.html"),
        format!(
            "<!doctype html><meta charset=utf-8><title>dev-tools</title>{PLACEHOLDER_SENTINEL}\
             <body><h1>dev-tools 前端未构建</h1>\
             <p>请在 web/ 下执行 <code>bun install &amp;&amp; bun run build</code>，\
             或安装 bun 后重新 <code>cargo build</code>。</p></body>"
        ),
    )?;
    Ok(())
}
