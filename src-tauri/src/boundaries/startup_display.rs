//! 启动后的自动加屏 / 自动切屏逻辑。
//!
//! 与 `fallback_display` 的区别：
//!   * fallback_display：快照驱动，**任何时候**发现没有活跃虚拟屏就补一块（持续兜底）。
//!   * 本模块：应用启动后**只做一次**，分两步、各自独立开关：
//!       1) `auto_add_display_on_launch`：延迟后若没有活跃虚拟屏则添加一块。
//!       2) `auto_switch_to_virtual_on_launch`：等虚拟屏 active 后 `DisplaySwitch /external`
//!          把桌面切到虚拟屏（仅第二屏），物理屏随之熄灭。
//!
//! 两者可独立开关，互不干扰。

use std::sync::Arc;
use std::time::Duration;

use tokio::time::sleep;

use crate::contracts::AppSettings;
use crate::host::StdioJsonRpcBackend;

/// 日志文件路径：放到 AppData 下（与 settings.json 同目录），
/// 不依赖 `C:\rustdesk-server` 这类临时/中间目录。
fn log_path() -> std::path::PathBuf {
    let base = std::env::var("APPDATA").unwrap_or_else(|_| ".".to_string());
    std::path::Path::new(&base)
        .join("com.easyvirtualdisplay.desktop")
        .join("vd_startup.log")
}

fn log_line(m: &str) {
    use std::io::Write;
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path())
    {
        let _ = writeln!(f, "[{}] {}", secs, m);
    }
}

/// 执行 `DisplaySwitch.exe /external`：只在第二屏（虚拟屏）显示，物理屏熄灭。
/// `/external` 是幂等的具体模式（不是切换开关），重复调用无害。
fn switch_to_virtual_display() -> bool {
    let sysroot = std::env::var("SystemRoot").unwrap_or_else(|_| r"C:\Windows".to_string());
    let ds = std::path::Path::new(&sysroot)
        .join("System32")
        .join("DisplaySwitch.exe");
    if !ds.exists() {
        log_line("[ERROR] 找不到 DisplaySwitch.exe，无法切到第二屏");
        return false;
    }
    match std::process::Command::new(&ds).arg("/external").spawn() {
        Ok(_) => {
            log_line("已执行 DisplaySwitch /external（桌面切到虚拟屏，物理屏熄灭）");
            true
        }
        Err(e) => {
            log_line(&format!("DisplaySwitch /external 失败: {:?}", e));
            false
        }
    }
}

/// 启动入口：根据设置，延迟后执行"加屏"和/或"切屏"，只跑一次。
pub fn spawn_startup_auto_add(backend: Arc<StdioJsonRpcBackend>, settings: &AppSettings) {
    let do_add = settings.auto_add_display_on_launch;
    let do_switch = settings.auto_switch_to_virtual_on_launch;
    let delay_secs = settings.auto_add_display_delay_secs;

    log_line(&format!(
        "spawn_startup_auto_add: add={} switch={} delay={}",
        do_add, do_switch, delay_secs
    ));

    if !do_add && !do_switch {
        return;
    }

    let delay = Duration::from_secs(delay_secs as u64);

    tauri::async_runtime::spawn(async move {
        sleep(delay).await;
        log_line("延迟结束, 开始执行启动任务");

        if do_add {
            add_display_if_needed(&backend).await;
        }

        if do_switch {
            switch_to_virtual_display_after_active(&backend).await;
        }
    });
}

/// 若没有活跃虚拟屏则添加一块。带重试，处理驱动管道偶发的 231(ERROR_PIPE_BUSY)。
async fn add_display_if_needed(backend: &Arc<StdioJsonRpcBackend>) {
    const MAX_ATTEMPTS: u32 = 6;
    for attempt in 1..=MAX_ATTEMPTS {
        match backend.refresh_snapshot().await {
            Ok(snapshot) => {
                let has_active = snapshot.displays.iter().any(|d| d.active);
                log_line(&format!(
                    "第{}次: 快照OK 屏数={} has_active={}",
                    attempt,
                    snapshot.displays.len(),
                    has_active
                ));
                if has_active {
                    log_line("已有活跃虚拟屏, 无需添加");
                    return;
                }
                match backend.add_display().await {
                    Ok(()) => {
                        log_line(&format!("第{}次: addDisplay 成功", attempt));
                        return;
                    }
                    Err(e) => log_line(&format!("第{}次: addDisplay 失败: {:?}", attempt, e)),
                }
            }
            Err(e) => log_line(&format!("第{}次: 快照失败: {:?}", attempt, e)),
        }
        sleep(Duration::from_secs(4)).await;
    }
    log_line("加屏重试耗尽, 放弃(不影响 App 其余功能)");
}

/// 等待虚拟屏在快照里变成 active（驱动插上并被系统枚举进桌面），再切换到"仅第二屏"。
/// 只有确认虚拟屏真的 active 才切，避免物理屏黑掉却没虚拟屏可看（锁死）。
async fn switch_to_virtual_display_after_active(backend: &Arc<StdioJsonRpcBackend>) {
    const MAX_WAIT_SECS: u32 = 30;
    let mut active = false;
    for _ in 0..(MAX_WAIT_SECS / 2) {
        sleep(Duration::from_secs(2)).await;
        match backend.refresh_snapshot().await {
            Ok(snapshot) => {
                let n = snapshot.displays.len();
                let a = snapshot.displays.iter().any(|d| d.active);
                log_line(&format!("等待虚拟屏 active: 屏数={} active={}", n, a));
                if a {
                    active = true;
                    break;
                }
            }
            Err(e) => log_line(&format!("等待虚拟屏 active: 快照失败 {:?}", e)),
        }
    }
    if !active {
        log_line("超时未等到虚拟屏 active, 不切换(保持物理屏点亮, 避免锁死)");
        return;
    }
    // 再等 3 秒让桌面/驱动完全就绪，然后切换。
    sleep(Duration::from_secs(3)).await;
    switch_to_virtual_display();
}
