use std::path::Path;
use std::process::Command;

fn main() -> anyhow::Result<()> {
    println!("cargo:rerun-if-changed=web/src");
    println!("cargo:rerun-if-changed=web/public");
    println!("cargo:rerun-if-changed=web/package.json");
    println!("cargo:rerun-if-changed=web/index.html");
    println!("cargo:rerun-if-changed=web/vite.config.ts");
    println!("cargo:rerun-if-changed=web/tsconfig.json");

    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let web = root.join("web");
    let dist = root.join("dist");

    if !web.exists() {
        std::fs::create_dir_all(&dist)?;
        ensure_placeholder(&dist)?;
        println!("cargo:warning=web/ 不存在，跳过前端构建（使用占位页）");
        return Ok(());
    }

    if which("bun").is_none() {
        std::fs::create_dir_all(&dist)?;
        ensure_placeholder(&dist)?;
        println!(
            "cargo:warning=未找到 bun，跳过前端构建。安装: curl -fsSL https://bun.sh/install | bash"
        );
        return Ok(());
    }

    if !web.join("node_modules").exists() {
        run("bun", &["install"], &web)?;
    }
    run("bun", &["run", "build"], &web)?;

    ensure_placeholder(&dist)?;
    Ok(())
}

fn which(cmd: &str) -> Option<std::path::PathBuf> {
    let output = Command::new("sh").arg("-c").arg(format!("command -v {cmd}")).output().ok()?;
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

fn ensure_placeholder(dist: &Path) -> anyhow::Result<()> {
    let index = dist.join("index.html");
    if !index.exists() {
        std::fs::write(
            &index,
            "<!doctype html><meta charset=utf-8><title>dev-tools</title>\
             <body><h1>dev-tools 前端未构建</h1>\
             <p>请在 web/ 下执行 <code>bun install &amp;&amp; bun run build</code>，\
             或安装 bun 后重新 <code>cargo build</code>。</p></body>",
        )?;
    }
    Ok(())
}
