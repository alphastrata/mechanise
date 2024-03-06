# ANTHROPIC SONNET (not even the 'good' one?)
## Sonnet:
### one-shot: 
- did not compile
- unused imports
- duplicate error impls
- limited clips (unused must use, really is the only issue.)
- great use of lifetimes
- great use of AnthropicError
- followed instructions 
- 10/10
- it just worked... wow.


### streaming:
- didn't update the error type
- hallucinated `next_line()` which doesn't exist.
- lack of error coercion caused a compile fail on the inline json from string it chose to create (note: not using a type -- just rawdawging it on the spot) -> not great.
```rust
    let event = serde_json::from_str::<StreamEvent>(&format!(r#"{{"type":"{}"}}"#, event_type))

```
- didn't update `Client` struct to add the streaming field.
>> After new input i.e giving it the compiler warning
- failed to update the error again for the utf8 parse
- statusCode not in namespace
- failed to Type<T> the buffer :
```sh
7 |         let mut buffer = Vec::new();
|             ^^^^^^^^^^   ---------- type must be known at this point
|
help: consider giving `buffer` an explicit type, where the type for type parameter `T` is specified
|
197 |         let mut buffer: Vec<T> = Vec::new();```
- Response is moved in the loop lols, so it's not seen that we don't impl copy.
- After many more turns, i got fed up and showed it the reqwest docs on streaming -- it failed to tell me to add the new dep `futures_util` or feature `stream` :(
- again missing statuscode and Stream that're not in scope.

# GPT4
### one-shot


### streaming	