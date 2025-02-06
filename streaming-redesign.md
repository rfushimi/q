## Streaming Redesign Design Doc

### 1. **Goals**

1. **Stable Realtime Output**  
   - Avoid cursor “jumping” and confusing terminal redraws.  
   - Cleanly present the streamed LLM response without the tool getting stuck or misplacing text.

2. **Clear Layout**  
   - Three lines at all times:  
     1. **Spinner + Status (dark gray) + Model Name (gray)**
     2. **Answer Text (green)**
     3. **Final “Done!” message (blue)**  

3. **Flexible Presentation**  
   - Allow future enhancements for colors, layout, and additional status lines.

### 2. **High-Level Approach**

1. **Use a Multi-Progress Interface**  
   - Instead of manually juggling ANSI cursor codes, rely on a “multi-progress” approach.  
   - **`indicatif::MultiProgress`** can manage several progress bars stacked vertically.  

2. **Separate Display Lines**  
   - **Line 1:** A **spinner** PB for showing ongoing status and model name.  
   - **Line 2:** A “progress bar” that we treat as a “text console.” We will print the streamed tokens into it.  
   - **Line 3:** A final progress bar or a final line used to show “Done!” in blue once streaming completes.

3. **Streaming Logic**  
   - We’ll gather tokens from the LLM as they arrive.  
   - Each chunk can be appended to a “buffer string,” which is then set on the second line.  
   - This approach ensures we keep a stable layout: the spinner stays in place on top, the output accumulates in the middle, and the final line remains at the bottom (initially empty or hidden).

4. **Color & Layout Control**  
   - We'll color tokens or lines as needed (e.g., green text for the LLM answer, dark gray for status).  
   - Because we’re using separate “progress bars,” we can style each line’s text independently without messing up the layout.

5. **Completion**  
   - Once we exhaust the stream, we finalize the second line (the LLM text) and set the third line to a short “Done!” message in blue.  
   - Stop the spinner on the first line or set it to a “finished” state with a final status message.

### 3. **Implementation Outline**

Below is a step-by-step plan using `indicatif::MultiProgress`:

1. **Create MultiProgress & Lines**  
   ```rust
   let multi = MultiProgress::new();

   // Line 1: Spinner + status
   let spinner = multi.add(ProgressBar::new_spinner());
   spinner.set_style(
       ProgressStyle::default_spinner()
           .template("{spinner:.green} {msg}")
           .unwrap()
   );
   spinner.set_message("Connecting... (model: gpt-4)");
   spinner.enable_steady_tick(std::time::Duration::from_millis(100));

   // Line 2: Text output
   let text_bar = multi.add(ProgressBar::new(0)); 
   // We won't really use the "length" aspect. We'll print tokens as log lines.
   text_bar.set_style(
       ProgressStyle::default_bar()
           .template("{msg}")
           .unwrap()
   );
   text_bar.set_message("");

   // Line 3: Completion
   let done_bar = multi.add(ProgressBar::new(1)); 
   done_bar.set_style(
       ProgressStyle::default_bar()
           .template("{msg}")
           .unwrap()
   );
   done_bar.set_message("");
   ```

2. **Spawn a Task to Drive MultiProgress**  
   - Because `MultiProgress` runs in a blocking fashion when using `.join()`, we can either:
     - **(A)** Spawn a blocking thread for `.join()`, or  
     - **(B)** Use the non-blocking “draw target” approach.  
   - Typically, you do something like:
     ```rust
     tokio::task::spawn_blocking(move || {
         multi.join().unwrap();
     });
     ```
   - This keeps the progress bars updating live while the async code continues to fetch tokens.

3. **Stream Tokens**  
   - Start receiving tokens from `LLMApi::send_streaming_query(...)` (a `Stream<Item = Result<String, ApiError>>`).  
   - Maintain a buffer `String` for the entire answer.  
   - For each token:
     ```rust
     // Add token to a local buffer
     answer_buffer.push_str(&token);

     // Color the token or entire line green
     let colored = format!("\x1B[32m{}\x1B[0m", answer_buffer);

     // Update the text_bar’s message
     text_bar.set_message(colored);
     ```
   - We do not rely on `inc()` or `finish()` for the text progress bar, because we’re effectively using it as a “live text area.”

4. **Final Steps**  
   1. When the stream completes normally:  
      - **Stop** the spinner (Line 1) with `spinner.finish_and_clear()`, or set a final message (e.g., “Finished”).  
      - **Set** the third line (`done_bar`) to “Done!” in blue:  
        ```rust
        done_bar.set_message("\x1B[34mDone!\x1B[0m");
        done_bar.finish();
        ```
   2. Handle errors by setting an error message on the final line, clearing or stopping the spinner, etc.

### 4. **Proposed File Structure**

We can keep a separate module in `src/core/streaming.rs`:

```rust
// src/core/streaming.rs
pub async fn handle_streaming_response(...) -> CoreResult<String> {
    // 1. Create MultiProgress and bars
    // 2. Spawn a blocking thread for multi.join()
    // 3. Send streaming query, read tokens in a loop
    // 4. Update the text_bar message
    // 5. On completion or error, finalize bars
    // 6. Return the final collected text
}
```

- This approach ensures all streaming concerns (progress bars, layout, etc.) are isolated.  
- The rest of the code only calls `handle_streaming_response(...)` to get the final text.

### 5. **Workflow Example**

1. **User runs**: `q "Hello?"`  
2. **Tool**:
   1. Sets up **spinner** on line 1: “Connecting… (model: gpt-4)”.  
   2. Starts reading from the streaming API.  
   3. Each time a token arrives, appends it to `answer_buffer` and updates line 2 with a green color.  
3. **When done**:  
   1. **Line 1** is cleared or changed to “Completed.”  
   2. **Line 3** is set to “Done!” in blue.  
4. **Output**: 3 stable lines (spinner line, answer line, done line). No cursor manipulation needed.

### 6. **Potential Pitfalls & Mitigations**

1. **Large Output**  
   - If the LLM returns huge output, storing it in `set_message()` repeatedly may degrade performance.  
   - Mitigation:  
     - Refresh the text only every N tokens or use partial flush.  
     - Or switch to a TUI library (like `crossterm`) if extremely large text is expected.  

2. **Async + MultiProgress**  
   - `MultiProgress::join()` is a blocking call. We must run it on a separate thread to avoid blocking the async runtime.  

3. **Terminal Incompatibility**  
   - On some terminals (especially Windows), certain ANSI sequences might not render perfectly.  
   - Indicatif handles most of this, but testing on multiple platforms is recommended.

4. **Customization**  
   - We want different colors, or more lines, or dynamic expansions in the future.  
   - By splitting out a dedicated `streaming.rs` module, we keep this flexible.

---

### Final Note

This design avoids complex cursor manipulation entirely, focusing on **indicatif**’s multi-progress capabilities. Each line is a separate “progress bar,” enabling stable, simple, color-coded streaming output. Once the LLM streaming finishes, we finalize everything with a concise “Done!” message.
