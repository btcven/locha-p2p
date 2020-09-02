// Copyright 2020 Bitcoin Venezuela and Locha Mesh Developers
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use async_std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

/// Task interrupt
///
/// A [`TaskInterrupt`] is atomically reference counted, so it can be cloned
/// and it will reference to the same [`TaskInterrupt`].
#[derive(Debug, Clone)]
pub struct TaskInterrupt {
    inner: Arc<(Mutex<bool>, Condvar)>,
}

impl TaskInterrupt {
    /// Create a new [`TaskInterrupt`]
    pub fn new() -> TaskInterrupt {
        TaskInterrupt {
            inner: Arc::new((Mutex::new(false), Condvar::new())),
        }
    }

    /// Interrupt the task.
    pub async fn interrupt(&self) {
        let &(ref lock, ref cvar) = &*self.inner;
        let mut flag = lock.lock().await;
        *flag = true;
        cvar.notify_all();
    }

    /// Sleep for the given time
    ///
    /// # Return
    ///
    /// - `true` if timed out
    /// - `false` if sleeping was interrupted
    pub async fn sleep_for(&self, dur: Duration) -> bool {
        let &(ref lock, ref cvar) = &*self.inner;

        let mut flag = lock.lock().await;
        let ret = cvar.wait_timeout(flag, dur).await;
        flag = ret.0;
        !(*flag)
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use std::thread;
    use std::time::Instant;

    #[test]
    fn test_task_sleeps_given_time() {
        let now = Instant::now();

        let time = Duration::from_millis(1000);

        let handle = thread::spawn({
            let time = time;
            move || {
                async_std::task::block_on(async {
                    let task_interrupt = TaskInterrupt::new();

                    task_interrupt.sleep_for(time).await;
                })
            }
        });

        handle.join().unwrap();

        let elapsed = now.elapsed();

        println!("time {}", elapsed.as_millis());
        // Check for elapsed time is +/- 50 ms
        assert!(
            elapsed >= (time - Duration::from_millis(50))
                && elapsed <= (time + Duration::from_millis(50))
        );
    }
}
