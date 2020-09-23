# Lieutenant
Lieutenant is a safe and fast deterministic command parsing libary.

## Command builder
```rust
CommandBuilder::new()
    .literal("tp")
    .inject()
    .param()
    .exec(|player: &mut Player, pos: Postion| {
        player.teleport(pos)?;
        player.send_message(text!({}))
    });
```

## DFA
Lieutenant uses a DFA for parsing, this makes parsing it `O(n) where n = len command_input`. This works by creating a jump table from each charecter to each next valid charecter. This have a memory cost of `O(nc`

