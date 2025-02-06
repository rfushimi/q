## Step-by-Step Tasks with Test Suggestions

Below is a breakdown of small, incremental steps to implement and verify the new streaming approach. We’ll use the **`indicatif::MultiProgress`** mechanism and spawn a blocking thread for rendering. We’ll provide guidance on how to test each step—both from a human perspective (actually seeing the terminal) and from the perspective of a code-based agent that cannot see the raw terminal output.

---

### **Task 1: Basic MultiProgress and Single Spinner**

1. **Create a new function** in your `streaming.rs` (or a test file) named `test_multibar_basics()`.  
2. **Initialize a `MultiProgress`** and add one spinner bar:
   ```rust
   let multi = MultiProgress::new();
   let spinner = multi.add(ProgressBar::new_spinner());
   spinner.set_style(
       ProgressStyle::default_spinner()
           .template("{spinner:.green} {msg}")
           .unwrap()
   );
   spinner.set_message("Testing spinner...");
   spinner.enable_steady_tick(std::time::Duration::from_millis(100));
   ```
3. **Spawn a blocking thread** to run `multi.join()`:
   ```rust
   let handle = std::thread::spawn(move || {
       multi.join().unwrap();
   });
   ```
4. **Let it spin for a couple seconds**, then call:
   ```rust
   std::thread::sleep(std::time::Duration::from_secs(2));
   spinner.finish_with_message("Spinner done!");
   handle.join().unwrap();
   ```
5. **Expected Behavior**:  
   - The spinner runs for ~2 seconds, then finishes with “Spinner done!”.
   - If you see a flickering spinner, it’s working.

#### **How to Verify**  
- **Human**: Run `cargo run --bin your_tool test_multibar_basics` (or however you integrate it). Visually confirm the spinner starts and then finishes.  
- **Agent**:  
  - Capture `stdout` (and possibly `stderr`) as a string buffer. Check if the final output contains the text “Spinner done!” or the spinner’s prefix.  
  - The agent can parse the final lines or logs for success markers.  

---

### **Task 2: Add a Second “Text” Bar**

1. **Add another progress bar** for text output. For now, just set a static message:
   ```rust
   let text_bar = multi.add(ProgressBar::new(0));
   text_bar.set_style(
       ProgressStyle::default_bar()
           .template("{msg}")
           .unwrap()
   );
   text_bar.set_message("Waiting for data...");
   ```
2. **In your code**:
   ```rust
   spinner.set_message("Spinner: loading...");
   text_bar.set_message("Initial text line");
   ```
3. **Run the same flow** (spawn thread, sleep 2s, finish).

#### **How to Verify**  
- **Human**: You should see two lines in the terminal:
  1. The spinner (Line 1)  
  2. The static “Initial text line” (Line 2)  
- **Agent**:  
  - Again, parse captured output. Look for “Spinner: loading...” and “Initial text line”.  

---

### **Task 3: Add a Third Bar for Completion**

1. **Add a third progress bar** for a final completion message:
   ```rust
   let done_bar = multi.add(ProgressBar::new(1));
   done_bar.set_style(ProgressStyle::default_bar().template("{msg}").unwrap());
   done_bar.set_message("");
   ```
2. **Finish logic**:
   ```rust
   // After 2s
   spinner.finish_and_clear();
   text_bar.set_message("Text line final state");
   done_bar.set_message("Done!");
   done_bar.finish_and_clear();
   ```
3. **Expected**:
   - After 2s, the spinner line is cleared, the text line updates, and the done line shows "Done!".

#### **How to Verify**  
- **Human**: Visually confirm that after 2s, you see something like:
  ```
  Text line final state
  Done!
  ```
- **Agent**:  
  - Parse output for “Text line final state” and “Done!”.  

---

### **Task 4: Introduce Streaming Logic with a Mock Stream**

1. **Create a mock stream** in your test or dev code that yields tokens over time:
   ```rust
   use futures::stream;
   use futures::StreamExt;
   use tokio::time::{sleep, Duration};

   let token_stream = stream::iter(vec!["Token1 ", "Token2 ", "Token3"])
       .then(|token| async move {
           sleep(Duration::from_millis(500)).await;
           Ok::<_, std::io::Error>(token)
       });
   ```
2. **Consume tokens** in an async function. For each token:
   ```rust
   let mut buffer = String::new();
   while let Some(Ok(chunk)) = token_stream.next().await {
       buffer.push_str(chunk);
       let colored = format!("\x1B[32m{}\x1B[0m", buffer); // green
       text_bar.set_message(colored);
   }
   ```
3. **Finish**:
   ```rust
   done_bar.set_message("\x1B[34mDone!\x1B[0m");
   done_bar.finish();
   spinner.finish_and_clear();
   ```

#### **How to Verify**  
- **Human**: You should see the second line gradually grow:
  - After 0.5s: “Token1 ”
  - After 1.0s: “Token1 Token2 ”
  - After 1.5s: “Token1 Token2 Token3”
  - Then a final line “Done!” in blue.  
- **Agent**:  
  - If capturing logs, your output should contain partial increments:
    - “Token1 ”
    - “Token1 Token2 ”
    - “Token1 Token2 Token3”
    - “Done!”  

---

### **Task 5: Integrate `spawn_blocking` in Actual CLI**

1. **In your real `handle_streaming_response()`**:
   1. Initialize your `MultiProgress` and bars.  
   2. Store `multi` in an `Arc` or move it into a blocking thread.  
   3. **Spawn** the multi-join thread:
      ```rust
      let m = multi.clone();
      let handle = tokio::task::spawn_blocking(move || {
          m.join().unwrap();
      });
      ```
   4. Perform the streaming in your async context.  
   5. Once done, finalize the bars.  
   6. **Wait** on the blocking thread with `handle.await.unwrap()`.

2. **Expected**:  
   - Your CLI call (`q "Hello?"`) should show the 3 lines as described, then finalize after the stream ends.

#### **How to Verify**  
- **Human**: Same visual checks.  
- **Agent**:  
  - Possibly run the CLI in a mode that captures output (or run a test harness that spawns the main).  
  - Parse final lines for success.

---

### **Task 6: Color and Layout Tweaks**

1. **Add color**:
   - For spinner line: Dark gray for the status, e.g., `"\x1B[90mConnecting...\x1B[0m"`.
   - For the second line: Already green for tokens.  
   - For the done line: Blue `"\x1B[34mDone!\x1B[0m"`.
2. **Style messages** with `ProgressStyle` or direct ANSI strings.

#### **How to Verify**  
- **Human**: Check that each line has the correct color.  
- **Agent**:  
  - If capturing terminal output, you’ll see ANSI escape codes. You can parse them or just confirm they’re present.

---

### **Task 7: Error Handling**  
1. **Simulate an error** mid-stream. For example, force a chunk to return an error:
   ```rust
   let token_stream = stream::iter(vec![
       Ok("Token1 "),
       Err("Simulated error!"),
       Ok("Token2 "),
   ]);
   ```
2. **When an error occurs**:
   - Set spinner to “Error!” and maybe color it red.  
   - Break out of the loop, finalize the bars.  

#### **How to Verify**  
- **Human**: You should see partial progress, then “Error!” message or a distinct final line.  
- **Agent**:  
  - Check logs or final lines for “Error!” or the error text.

---

### **Final Notes for an Automated Agent**

- **Capture Output**: Most CI systems, test harnesses, or “agents” can capture `stdout` and `stderr`.  
- **Parse Lines**: While you lose the real-time visual, you can confirm the final text block contains the expected tokens in order.  
- **Intermediate States**: If you want to confirm partial states, you can forcibly flush logs or add debug lines. For example, each time you call `text_bar.set_message(...)`, also do a `println!("[DEBUG]: new message = {}", buffer)` so the agent can parse the debug lines in real time.  

By following these incremental tasks and verifying at each step—visually or via logs—you should isolate any issues early and confirm that the final multi-progress approach works as intended.
