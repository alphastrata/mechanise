# ANTHROPIC SONNET (not even the 'good' one?)
## Sonnet:
### one-shot: 
- did not compile
- unused imports
- duplicate error `impls`
- limited clips (unused must use, really is the only issue.)
- great use of `lifetimes`
- great use of `AnthropicError`
- followed instructions 
With very minimal intervention it worked... wow.
9/10 and in total minus typing the prompt out this took less than 10 mins, that is -- for the most part in _most_ universes such an incredible win.

[code here](linkTODO)

### streaming:
 Throwing in the now working non-streaming version, I wanted to see if the model could adapt its own code && implement what was required.

- didn't update the error type
- hallucinated `next_line()` which doesn't exist.
- lack of error coercion caused a compile fail on the inline `json` from string it chose to create (note: not using a type -- just raw-dawging it on the spot) -> not great.
```rust
    let event = serde_json::from_str::<StreamEvent>(&format!(r#"{{"type":"{}"}}"#, event_type))

```
- didn't update `Client` struct to add the streaming field.
>> After new input i.e giving it the compiler warning
- failed to update the error again for the `utf8` parse
- `StatusCode` not in namespace
- failed to Type<T> the buffer :
```sh
7 |         let mut buffer = Vec::new();
|             ^^^^^^^^^^   ---------- type must be known at this point
|
help: consider giving `buffer` an explicit type, where the type for type parameter `T` is specified
|
197 |         let mut buffer: Vec<T> = Vec::new();
```
- Response is moved in the loop, so it's not seen that we don't impl copy.
- After many more turns, i got fed up and showed it the reqwest docs on streaming -- it failed to tell me to add the new dep `futures_util` or feature `stream` :(
- again missing `StatusCode` and `Stream` that're not in scope.

---
response cap hit multiple times in the back and forth -- I'd give Sonnet an F for this  extended version of the task.
---
So Sonnet actually didn't end up getting the streaming to work without heavy intervention,
the big takeaway was that the responses are not json, which is wasn't accounting for:

```json
event: message_start
data: {"type":"message_start","message":{"id":"..SNIPPING..","type":"message","role":"assistant","content":[],"model":"claude-3-opus-20240229","stop_reason":null,"stop_sequence":null,"usage":{"input_tokens":10,"output_tokens":1}}}

event: content_block_start
data: {"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}

event: ping
data: {"type": "ping"}

event: content_block_delta
data: {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"Hello"}}

event: content_block_delta
data: {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"!"}}

event: content_block_stop
data: {"type":"content_block_stop","index":0}

event: message_delta
data: {"type":"message_delta","delta":{"stop_reason":"max_tokens","stop_sequence":null},"usage":{"output_tokens":2}}

event: message_stop
data: {"type":"message_stop"}
```

So what the model should've done was something like this:
```rust
// Firstly:
if let Some(data) = line.strip_suffix("data: ") { ... snip ... }
```
which lets them ignore the event line entirely (because `serde` clever using the `tag` {which to Sonnet's credit it DID have there}) see [docs](https://serde.rs/enum-representations.html):
```rust
#[serde(tag = "type", rename_all = "snake_case")]
enum StreamEvent{ ... snip ... }
```
then because the `if let` is doing all the heavy lifting:
```rust
// Secondly
serde_json::from_str::<StreamEvent>(data)?;
```
Would've done the trick and then you can do your own match etc to pull the `.text` fields out of them etc etc.

Unfortunately with _a lot_ of prompting despite being led to this issue the model gave us this tripe:
```rust
    let mut current_event_type = String::new();
    let mut current_data = String::new();

    while let Some(line) = lines.next_line().await? {
        if line.starts_with("event: ") {
            if !current_event_type.is_empty() && !current_data.is_empty() {
                // Attempt to parse the current event
                parse_event(&current_event_type, &current_data)?;
                // Reset for the next event
                current_event_type.clear();
                current_data.clear();
            }
            current_event_type = line["event: ".len()..].to_string();
        } else if line.starts_with("data: ") {
            current_data = line["data: ".len()..].to_string();
        }
    }

    // Don't forget to process the last event if the file ends right after a data line
    if !current_event_type.is_empty() && !current_data.is_empty() {
        parse_event(&current_event_type, &current_data)?;
    }
```
The problems **should** be quite evident here, however a non-Rust language may need this level of verbosity due to a lack of sophisticated libraries like `serde` and so on.

Note also the continued inappropriate use of the slicing when we have `strip_prefix` :(.

What is clear from this is that the model doesn't know what it's regurgitated to us.
It doesn't understand the behaviour of what `serde`'s `tag` does, it doesn't learn from the corrections given to it by me when re-fed into the new request's context window and it is failing to, despite all help and aid been given to the contrary appreciate that we can greatly reduce the nesting and checking here as we _only_ care about data. (because `serde` is already going to handle the permutations of what the `event` can be parsed into anyway.)




# GPT4
### one-shot


### streaming	

