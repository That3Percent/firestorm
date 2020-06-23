use {
    crate::internal::*,
    firestorm_core::*,
    inferno::flamegraph,
    std::{
        collections::HashMap,
        time::{Duration, Instant},
    },
};

// TODO: Consider fancier parsing rules
#[macro_export]
macro_rules! profile_fn {
    ($($t:tt)*) => {
        let _firestorm_fn_guard = { $crate::internal::LazyStr::Func(stringify!($($t:tt)*)) };
    };
}

#[macro_export]
macro_rules! profile_method {
    ($($t:tt)*) => {
        let _firestorm_method_guard = { $crate::internal::LazyStr::Method(::std::any::type_name::<Self>(), stringify!($($t:tt)*)) };
    };
}

#[inline(always)]
#[must_use = "Use a let binding to extend the lifetime of the guard to the scope which to profile."]
pub fn profile_scope(name: &'static str) -> impl Drop {
    start(LazyStr::Func(name));
    SpanGuard
}

/// Clears all of the recorded info that firestorm has tracked in this thread.
pub fn clear() {
    with_events(|e| e.clear());
}

/// Convert events to the format that inferno is expecting
fn lines(options: &Options) -> Vec<String> {
    with_events(|events| {
        struct Frame {
            name: String,
            start: Instant,
        }
        struct Line {
            name: String,
            time: u128,
        }

        fn push_line(lines: &mut Vec<Line>, name: String, duration: Duration) {
            let time = duration.as_nanos();
            if let Some(prev) = lines.last_mut() {
                if &prev.name == &name {
                    prev.time += time;
                    return;
                }
            }
            lines.push(Line { name, time });
        }

        let mut stack = Vec::<Frame>::new();
        let mut collapsed = HashMap::<_, u128>::new();
        let mut lines = Vec::<Line>::new();

        for event in events.iter() {
            match event {
                Event::Start { time, tag } => {
                    let mut name = format!("{}", tag).replace(";", "").replace(" ", "_");
                    if let Some(parent) = stack.last() {
                        name = format!("{};{}", &parent.name, name);
                        if !options.merge {
                            push_line(&mut lines, name.clone(), *time - parent.start);
                        }
                    }
                    let frame = Frame {
                        name: name,
                        start: *time,
                    };
                    stack.push(frame);
                }
                Event::End { time } => {
                    let Frame { name, start } = stack.pop().unwrap();
                    let elapsed = *time - start;
                    if options.merge {
                        let elapsed = elapsed.as_nanos();
                        let entry = collapsed.entry(name).or_default();
                        *entry = entry.wrapping_add(elapsed);
                        if let Some(parent) = stack.last() {
                            let entry = collapsed.entry(parent.name.clone()).or_default();
                            *entry = entry.wrapping_sub(elapsed);
                        }
                    } else {
                        push_line(&mut lines, name, elapsed);
                        if let Some(parent) = stack.last_mut() {
                            parent.start = *time;
                        }
                    }
                }
            }
        }
        assert!(stack.is_empty(), "Mimatched start/end");

        fn format_line(name: &str, time: &u128) -> Option<String> {
            if *time == 0 {
                None
            } else {
                Some(format!("{} {}", name, time))
            }
        }

        if options.merge {
            collapsed
                .iter()
                .filter_map(|(name, time)| format_line(name, time))
                .collect()
        } else {
            lines
                .iter()
                .filter_map(|Line { name, time }| format_line(name, time))
                .collect()
        }
    })
}

/// This API is unstable, and will likely go away
#[derive(Default)]
struct Options {
    /// Merges all instances of the same stack into a single bar.
    /// This may give a better overview of, for example, how much total
    /// time a method took. But, will not retain information like how
    /// many times a method was called.
    pub merge: bool,
    _priv: (),
}

/// Write the data to an svg
pub fn to_svg<W: std::io::Write>(writer: &mut W) -> Result<(), impl std::error::Error> {
    let options = Default::default();

    let lines = lines(&options);

    /*
    // Output lines for debugging
    use std::io::Write;
    let mut f = std::fs::File::create("C:\\git\\flames.txt").unwrap();
    for line in lines.iter() {
        f.write(line.as_bytes()).unwrap();
        f.write("\n".as_bytes()).unwrap();
    }
    drop(f);
    */

    let mut fg_opts = flamegraph::Options::default();
    fg_opts.count_name = "ns".to_owned();
    fg_opts.hash = true;
    fg_opts.flame_chart = !options.merge;
    flamegraph::from_lines(&mut fg_opts, lines.iter().rev().map(|s| s.as_str()), writer)
}

/// Finish profiling a section.
pub(crate) fn end() {
    with_events(|e| {
        e.push(Event::End {
            time: Instant::now(),
        })
    });
}

/// Unsafe! This MUST not be used recursively
/// TODO: Verify in Debug this is not used recursively
fn with_events<T>(f: impl FnOnce(&mut Vec<Event>) -> T) -> T {
    EVENTS.with(|e| {
        let r = unsafe { &mut *e.get() };
        f(r)
    })
}



/// Starts profiling a section. Every start MUST be followed by a corresponding
/// call to end()
pub(crate) fn start(tag: LazyStr) {
    with_events(|events| {
        let event = Event::Start {
            time: Instant::now(),
            tag,
        };
        events.push(event);
    });
}
