//! Timer Service Example
//! 定时器服务示例
//!
//! A service that demonstrates timer usage with scheduled tasks.
//! 演示定时器使用和计划任务的服务。
//!
//! Run with: cargo run --example runtime-timer-service
//! 运行: cargo run --example runtime-timer-service

use hiver_runtime::{Runtime, spawn, sleep, sleep_until, Duration, Instant};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ScheduledTask {
    id: usize,
    name: String,
    interval: Duration,
    last_run: Option<Instant>,
    run_count: usize,
}

struct TimerService {
    tasks: Arc<Mutex<HashMap<usize, ScheduledTask>>>,
    next_id: Arc<Mutex<usize>>,
}

impl TimerService {
    fn new() -> Self {
        Self {
            tasks: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(0)),
        }
    }

    fn schedule_task(&self, name: String, interval: Duration) -> usize {
        let mut next_id = self.next_id.lock().unwrap();
        let id = *next_id;
        *next_id += 1;

        let task = ScheduledTask {
            id,
            name,
            interval,
            last_run: None,
            run_count: 0,
        };

        let mut tasks = self.tasks.lock().unwrap();
        tasks.insert(id, task);
        id
    }

    fn run_task(&self, id: usize) {
        let mut tasks = self.tasks.lock().unwrap();
        if let Some(task) = tasks.get_mut(&id) {
            task.last_run = Some(Instant::now());
            task.run_count += 1;
            tracing::info!(
                "Task '{}' (id: {}) executed (count: {})",
                task.name,
                task.id,
                task.run_count
            );
            tracing::info!(
                "任务 '{}' (id: {}) 已执行 (次数: {})",
                task.name,
                task.id,
                task.run_count
            );
        }
    }

    fn get_task(&self, id: usize) -> Option<ScheduledTask> {
        let tasks = self.tasks.lock().unwrap();
        tasks.get(&id).cloned()
    }

    fn list_tasks(&self) -> Vec<ScheduledTask> {
        let tasks = self.tasks.lock().unwrap();
        tasks.values().cloned().collect()
    }
}

fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let mut runtime = Runtime::new()?;
    let service = Arc::new(TimerService::new());

    runtime.block_on(async {
        tracing::info!("Timer Service started");
        tracing::info!("定时器服务已启动");
        tracing::info!("");

        // Schedule multiple tasks with different intervals
        // 安排多个具有不同间隔的任务
        let task1_id = service.schedule_task("Health Check".to_string(), Duration::from_secs(5));
        let task2_id = service.schedule_task("Data Sync".to_string(), Duration::from_secs(10));
        let task3_id = service.schedule_task("Cleanup".to_string(), Duration::from_secs(15));

        tracing::info!("Scheduled tasks:");
        tracing::info!("已安排任务:");
        for task in service.list_tasks() {
            tracing::info!(
                "  - {} (id: {}, interval: {:?})",
                task.name,
                task.id,
                task.interval
            );
        }
        tracing::info!("");

        // Spawn task runners
        // 生成任务运行器
        let service1 = service.clone();
        spawn(async move {
            let mut next_run = Instant::now() + Duration::from_secs(5);
            loop {
                sleep_until(next_run).await;
                service1.run_task(task1_id);
                next_run = Instant::now() + Duration::from_secs(5);
            }
        });

        let service2 = service.clone();
        spawn(async move {
            let mut next_run = Instant::now() + Duration::from_secs(10);
            loop {
                sleep_until(next_run).await;
                service2.run_task(task2_id);
                next_run = Instant::now() + Duration::from_secs(10);
            }
        });

        let service3 = service.clone();
        spawn(async move {
            let mut next_run = Instant::now() + Duration::from_secs(15);
            loop {
                sleep_until(next_run).await;
                service3.run_task(task3_id);
                next_run = Instant::now() + Duration::from_secs(15);
            }
        });

        // Monitor tasks
        // 监控任务
        spawn(async move {
            loop {
                sleep(Duration::from_secs(20)).await;
                tracing::info!("");
                tracing::info!("Task Status Report:");
                tracing::info!("任务状态报告:");
                for task in service.list_tasks() {
                    let status = if let Some(last_run) = task.last_run {
                        format!("Last run: {:?} ago", last_run.elapsed())
                    } else {
                        "Not run yet".to_string()
                    };
                    tracing::info!(
                        "  {}: {} runs, {}",
                        task.name,
                        task.run_count,
                        status
                    );
                }
                tracing::info!("");
            }
        });

        // Run for 60 seconds
        // 运行60秒
        tracing::info!("Service will run for 60 seconds...");
        tracing::info!("服务将运行60秒...");
        sleep(Duration::from_secs(60)).await;

        tracing::info!("");
        tracing::info!("Final Task Statistics:");
        tracing::info!("最终任务统计:");
        for task in service.list_tasks() {
            tracing::info!(
                "  {}: {} total runs",
                task.name,
                task.run_count
            );
        }
    })
}
