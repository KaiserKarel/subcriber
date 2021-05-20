## Building

Since ink! does not use features in an additive way, your compilation cache can get corrupted. Try 

```
cargo clean
```

and then rerunning build/test related commands. (Specifically, running `cargo test` and then `cargo contract build`) may
lead to problems.