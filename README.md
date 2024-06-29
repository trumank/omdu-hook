# omdu-hook
Hooks UWorld::Listen stub to re-implement necessary listen server logic.

## building
```
# most recent version
cargo build --release --features=manifest-4932913164832566208

# or 808827202674972462
cargo build --release --features=manifest-808827202674972462
```

## usage
Copy `target/release/omdu_hook.dll` to `OrcsMustDieUnchained/Binaries/Win64/x3daudio1_7.dll`
