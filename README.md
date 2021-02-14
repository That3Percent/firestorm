Firestorm is the low-overhead intrusive flamegraph profiler for Rust.

[Tenets](#tenets) of the design

For instrumenting start [here](#instrumenting).

For application profiling start [here](#profiling).

# Tenets of the design
Firestorm is performance. When you are writing a library that is the fastest in it's class, you need a profiler that has the same ideals. When Firestorm is not enabled (the default) all invocations compile down to a no-op. When Firestorm is enabled, it makes no heap allocations and avoids as much work as possible in the critical section. 

Firestorm is ubiquitous. When Firestorm is enabled by the application layer for profiling, it is enabled for all dependencies - transitively. This happens without needing to add feature flags to libraries or even be aware which libraries use Firestorm. By avoiding making Firestorm a part of a library's public API, combined with the fact that Firestorm compiles down to a no-op when not used... adding Firestorm to a library should be a no-brainer. This is good because applications benefit as more libraries adopt Firestorm.

Firestorm is insight. After profiling, Firestorm offers three different ways to view the data - each looking at the performance of the run from a different angle.

# Instrumenting

First, add Firestorm to your dependencies in Cargo.toml:

```toml
[dependencies]
firestorm = "0.4"
```

Then, import Firestorm's profiling macros. I recommend doing this in a crate `prelude` module.

```rust
pub(crate) use firestorm::{
    profile_fn,
    profile_method,
    profile_section
};
```

Lastly, invoke the macros in your functions.

```rust
fn fn_name<T>(param: T) {
    // You can optionally add generic parameters
    profile_function!(T, fn_name);

    // If a function is complex, profile a section.
    {
        profile_section!(inner);

        // Optional: manually drop.
        // Section automatically drops when going out of scope.
        drop(inner);
    }
}

fn method_name(&self) {
    // profile_method automatically captures the type of Self
    profile_method!(method_name);
}
```

Important tips for instrumenting libraries:
 * Do NOT target an exact version of Firestorm. Eg: Do not use `firestorm = "=0.4.1"`. Use `firestorm = "0.4.1"` instead. This is an important part of Firestorm's backward compatibility policy. If `firestorm-core` needs to be updated, all major versions will receive a patch so that all versions of Firestorm are enabled. Targeting a specific version may prevent libraries from sharing the `firestorm-core` dependency or being enabled transitively.
 * Do NOT put firestorm in `[dev-dependencies]`. Always put firestorm in `[dependencies]`.
 * Do NOT enable any firestorm features in your library code, such as `enable_system_time`. This will prevent Firestorm from compiling down to a no-op when not in use.
 * Do NOT hide the use of Firestorm behind a feature flags or `[cfg()]`. Enabling/Disabling Firestorm is not a part of your public API and should always be enabled.


# Profiling