# Collections which transparently compress data to reduce memory usage

This create offers collections which automatically compress themselves, which reduces memory usage, sometimes
substantially, allowing collections to be held in memory that would otherwise be too big.

The only restriction on the datatypes in the collections is that they must be serde serializable.

For instance:
```
use compressed_collections::Stack;

let mut compressed_stack = Stack::new();
for _ in 0..(1024 * 1024 * 100) {
    compressed_stack.push(1);
}
```
This only allocates around 10MB (the default buffer size), whereas the equivalent vector would be around 100MB in size.

Design goals:
- Provide collections with a subset of the API of the standard equivalent wherever possible for easy dropin use.
- Only implement the efficient operations for each datastructure

Datastructures:
- [x] Stack
- [x] Deque
- [ ] Map

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.