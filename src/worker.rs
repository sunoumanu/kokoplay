use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{channel, Sender};
use std::sync::Arc;

pub struct WorkerHandle {
    pub tx: Sender<String>,
    pub queue_len: Arc<AtomicUsize>,
}

/// Spawns a dedicated OS thread that drains `text` items one at a time,
/// calling `process` for each. Mirrors the Python worker thread: a panic or
/// error from `process` is logged and the worker keeps running -- it must
/// never die, or the queue backs up forever with no visible symptom besides
/// requests silently never being spoken.
pub fn spawn_worker<F>(process: F) -> WorkerHandle
where
    F: Fn(&str) -> Result<(), Box<dyn std::error::Error>> + Send + 'static,
{
    let (tx, rx) = channel::<String>();
    let queue_len = Arc::new(AtomicUsize::new(0));
    let queue_len_worker = queue_len.clone();

    std::thread::spawn(move || {
        for text in rx {
            queue_len_worker.fetch_sub(1, Ordering::SeqCst);
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| process(&text)));
            match result {
                Ok(Ok(())) => {}
                Ok(Err(e)) => eprintln!("TTS playback failed: {e}"),
                Err(_) => eprintln!("TTS playback panicked and was recovered; worker continues"),
            }
        }
    });

    WorkerHandle { tx, queue_len }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use std::time::Duration;

    fn wait_until(mut check: impl FnMut() -> bool) {
        for _ in 0..200 {
            if check() {
                return;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        panic!("condition not met in time");
    }

    #[test]
    fn processes_items_serially_and_survives_errors() {
        let calls: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let calls_clone = calls.clone();
        let handle = spawn_worker(move |text| {
            calls_clone.lock().unwrap().push(text.to_string());
            if text == "fail" {
                return Err("boom".into());
            }
            Ok(())
        });

        handle.tx.send("fail".to_string()).unwrap();
        handle.tx.send("ok".to_string()).unwrap();

        wait_until(|| calls.lock().unwrap().len() == 2);
        assert_eq!(*calls.lock().unwrap(), vec!["fail", "ok"]);
    }

    #[test]
    fn survives_panics() {
        let calls: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let calls_clone = calls.clone();
        let handle = spawn_worker(move |text| {
            if text == "panic" {
                panic!("boom");
            }
            calls_clone.lock().unwrap().push(text.to_string());
            Ok(())
        });

        handle.tx.send("panic".to_string()).unwrap();
        handle.tx.send("after".to_string()).unwrap();

        wait_until(|| calls.lock().unwrap().len() == 1);
        assert_eq!(*calls.lock().unwrap(), vec!["after"]);
    }

    #[test]
    fn queue_len_decrements_on_receive_not_on_completion() {
        let (release_tx, release_rx) = channel::<()>();
        let release_rx = Arc::new(Mutex::new(release_rx));
        let handle = spawn_worker(move |_text| {
            release_rx.lock().unwrap().recv().ok();
            Ok(())
        });

        handle.queue_len.fetch_add(1, Ordering::SeqCst);
        handle.tx.send("blocked".to_string()).unwrap();

        wait_until(|| handle.queue_len.load(Ordering::SeqCst) == 0);

        release_tx.send(()).unwrap();
    }
}
