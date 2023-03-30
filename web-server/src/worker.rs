use std::process::Stdio;

use anyhow::{anyhow, bail, Context, Result};
use log::info;
use serde::Deserialize;
use serde_json::json;
use std::sync::Mutex;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    process::{Child, ChildStdout, Command},
};
#[derive(Deserialize, Debug)]
struct QueryResponse {
    reply: String,
    ok: bool,
}

pub struct Worker {
    child: Child,
    child_stdout: BufReader<ChildStdout>,
    started: bool,
    py_path: String,
    args: Vec<String>,
}
impl Worker {
    pub fn new(py_path: impl Into<String>, args: &[String]) -> Result<Self> {
        let py_path = py_path.into();
        let mut worker_process = Command::new(&py_path)
            .args(args)
            .stdout(Stdio::piped())
            .stdin(Stdio::piped())
            .stderr(Stdio::inherit())
            .kill_on_drop(true)
            .spawn()?;
        let stdout = worker_process.stdout.take().unwrap();
        Ok(Self {
            child: worker_process,
            child_stdout: BufReader::new(stdout),
            started: false,
            args: args.to_vec(),
            py_path,
        })
    }
    fn check_alive(&mut self) -> Result<()> {
        if let Ok(Some(v)) = self.child.try_wait() {
            info!("Worker died with exit code: {:?}, restarting..", v);
            *self = Self::new(self.py_path.clone(), &self.args)
                .map_err(|e| anyhow!("Failed to restart: {}", e))?;
        }
        Ok(())
    }
    pub async fn query(&mut self, query: impl Into<String>) -> Result<String> {
        if !self.started {
            self.wait_to_start().await?;
        }
        self.check_alive()?;
        let stdin = self.child.stdin.as_mut().unwrap();
        let json_string = json!({
            "task": "query",
            "query": query.into()
        })
        .to_string();
        info!("Querying: {}", json_string);
        stdin
            .write_all(format!("{}\n", json_string).as_bytes())
            .await?;
        stdin.flush().await?;
        let resp_line = self.poll_line().await?;
        let resp_struct = serde_json::from_str::<QueryResponse>(
            &String::from_utf8(resp_line)
                .with_context(|| anyhow!("Invalid utf8 chars encountered"))?,
        )
        .with_context(|| anyhow!("Malformed json encountered"))?;
        info!("Response: {:?}", resp_struct);
        if resp_struct.ok {
            Ok(resp_struct.reply)
        } else {
            bail!("Failed to query: {}", resp_struct.reply)
        }
    }
    pub async fn wait_to_start(&mut self) -> Result<()> {
        if self.started {
            bail!("Already started");
        }
        let line = self.poll_line().await?;

        match self.child.try_wait()? {
            Some(code) => {
                bail!("Worker failed to start: {}", code)
            }
            None => {
                if line == b"done" {
                    self.started = true;
                    Ok(())
                } else {
                    bail!("Worker failed to start with: {:?}", line);
                }
            }
        }
    }
    async fn poll_line(&mut self) -> Result<Vec<u8>> {
        let mut out_buf = vec![];
        self.child_stdout.read_until(b'\n', &mut out_buf).await?;
        while !out_buf.is_empty() && out_buf.last().map(|v| *v == b'\n' || *v == b'\r').unwrap() {
            out_buf.pop();
        }
        Ok(out_buf)
    }
}

pub struct WorkerManager {
    using_flags: Mutex<Vec<bool>>,
    workers: Vec<Mutex<Worker>>,
}
impl WorkerManager {
    pub fn new(worker_count: usize, python_executable: &str, worker_py: &str) -> Result<Self> {
        let using_flags = Mutex::new(vec![false; worker_count]);
        let mut workers = vec![];
        for _ in 0..worker_count {
            workers.push(Mutex::new(Worker::new(
                python_executable,
                &[worker_py.to_string()],
            )?));
        }
        Ok(Self {
            using_flags,
            workers,
        })
    }
    pub async fn wait_for_all_start(&mut self) -> Result<()> {
        for item in self.workers.iter_mut() {
            // It won't panic, since we haven't touched the workers yet.
            item.get_mut().unwrap().wait_to_start().await?;
        }
        Ok(())
    }
    pub async fn run_query(&self, query: impl Into<String>) -> Result<String> {
        let guard = UsingGuard::new(&self.using_flags)?;
        let curr_worker = &self.workers[guard.get_locked_idx()];
        let mut worker_guard =
            { curr_worker.lock() }.map_err(|e| anyhow!("Failed to lock: {}", e))?;

        let resp = worker_guard.query(query).await?;
        Ok(resp)
    }
}

/// A helper object to help us occupy a worker.
/// It will set the usable flag of a worker to false, and set that to true when being dropped
pub struct UsingGuard<'a> {
    lock: &'a Mutex<Vec<bool>>,
    idx: usize,
}

impl<'a> UsingGuard<'a> {
    pub fn get_locked_idx(&self) -> usize {
        self.idx
    }
    pub fn new(lock: &'a Mutex<Vec<bool>>) -> Result<UsingGuard<'a>> {
        let mut guard = lock.lock().map_err(|e| anyhow!("{}", e))?;
        let mut empty_slot = None;
        for (i, using_ref) in guard.iter_mut().enumerate() {
            if !(*using_ref) {
                empty_slot = Some((i, using_ref));
                break;
            }
        }
        if let Some((idx, usable_ref)) = empty_slot {
            *usable_ref = true;
            Ok(Self { lock, idx })
        } else {
            bail!("No available worker found. Wait for a moment and retry.");
        }
    }
}
impl<'a> Drop for UsingGuard<'a> {
    fn drop(&mut self) {
        self.lock.lock().unwrap()[self.idx] = false;
    }
}

#[cfg(test)]
#[allow(clippy::bool_assert_comparison)]
mod test {

    use std::{
        sync::{Arc, Mutex},
        time::Duration,
    };

    use tokio::sync::{watch, Semaphore};

    use crate::worker::UsingGuard;

    #[tokio::test]
    async fn test_using_guard_1() {
        let lock = Mutex::new(vec![false; 4]);
        let guard1 = UsingGuard::new(&lock).unwrap();
        assert_eq!(lock.try_lock().unwrap().iter().filter(|v| **v).count(), 1);
        let using_idx = guard1.get_locked_idx();
        assert_eq!(lock.try_lock().unwrap()[using_idx], true);
        drop(guard1);
        assert!(lock.try_lock().unwrap().iter().all(|v| !(*v)));
        assert_eq!(lock.try_lock().unwrap()[using_idx], false);
    }
    #[tokio::test]
    async fn test_using_guard_2() {
        let lock = Mutex::new(vec![false; 4]);
        let guard1 = UsingGuard::new(&lock).unwrap();
        let guard2 = UsingGuard::new(&lock).unwrap();
        assert_eq!(lock.try_lock().unwrap().iter().filter(|v| **v).count(), 2);
        let idx1 = guard1.get_locked_idx();
        let idx2 = guard2.get_locked_idx();
        assert_eq!(lock.try_lock().unwrap()[idx1], true);
        assert_eq!(lock.try_lock().unwrap()[idx2], true);

        drop(guard1);
        assert_eq!(lock.try_lock().unwrap()[idx1], false);
        assert_eq!(lock.try_lock().unwrap()[idx2], true);

        drop(guard2);
        assert_eq!(lock.try_lock().unwrap()[idx1], false);
        assert_eq!(lock.try_lock().unwrap()[idx2], false);
    }
    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_using_guard_3() {
        let lock_ptr = Arc::new(Mutex::new(vec![false; 4]));
        let mut handles = vec![];
        let sema = Arc::new(Semaphore::new(4));
        let (tx, rx) = watch::channel(false);
        for _ in 0..4 {
            let lock_ptr = lock_ptr.clone();
            let sema = sema.clone();
            let mut rx = rx.clone();
            handles.push(tokio::spawn(async move {
                let guard = UsingGuard::new(&lock_ptr).unwrap();

                assert_eq!(lock_ptr.lock().unwrap()[guard.get_locked_idx()], true);
                std::mem::forget(sema.acquire_owned().await.unwrap());
                rx.changed().await.unwrap();
            }));
        }
        // Let it spin, and wait for all tasks going to sleep
        while sema.available_permits() != 0 {
            tokio::time::sleep(Duration::from_millis(1)).await;
        }
        // So here all tasks are waiting for the watch's changing
        assert_eq!(lock_ptr.lock().unwrap().iter().all(|v| *v), true);
        // Tests done. Let all tasks continue
        tx.send(true).unwrap();
        for handle in handles.into_iter() {
            handle.await.unwrap();
        }
        assert_eq!(lock_ptr.lock().unwrap().iter().all(|v| !(*v)), true);
    }
}
