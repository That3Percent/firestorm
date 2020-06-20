#[cfg(feature = "enabled")]
extern crate inferno;
#[cfg(feature = "enabled")]
use inferno::flamegraph;
#[cfg(feature = "enabled")]
use std::cell::UnsafeCell;
#[cfg(feature = "enabled")]
use std::collections::HashMap;
#[cfg(feature = "enabled")]
use std::fmt;
#[cfg(feature = "enabled")]
use std::time::{Duration, Instant};

// TODO: Feature to enable, so that all crates that use it are consistently enabled

#[cfg(feature = "enabled")]
thread_local!(static EVENTS: UnsafeCell<Vec<Event>> = UnsafeCell::new(Vec::with_capacity(2048)));

#[cfg(feature = "enabled")]
/// Unsafe! This MUST not be used recursively
/// TODO: Verify in Debug this is not used recursively
fn with_events<T>(f: impl FnOnce(&mut Vec<Event>) -> T) -> T {
    EVENTS.with(|e| {
        let r = unsafe { &mut *e.get() };
        f(r)
    })
}

/// A delayed formatting struct to move allocations out of the loop
/// This API is likely to change
#[non_exhaustive]
pub enum FmtStr {
    Str1(&'static str),
    Str2(&'static str, &'static str),
    Str3(&'static str, &'static str, &'static str),
    StrNum(&'static str, u64),
}

impl From<&'static str> for FmtStr {
    #[inline(always)]
    fn from(value: &'static str) -> Self {
        Self::Str1(value)
    }
}

#[cfg(feature = "enabled")]
impl fmt::Display for FmtStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Str1(s0) => write!(f, "{}", s0),
            Self::Str2(s0, s1) => write!(f, "{}{}", s0, s1),
            Self::Str3(s0, s1, s2) => write!(f, "{}{}{}", s0, s1, s2),
            Self::StrNum(s0, n1) => write!(f, "{}{}", s0, n1),
        }
    }
}

#[cfg(feature = "enabled")]
/// A tiny record of a method call which when played back can construct
/// a profiling state.
enum Event {
    Start { time: Instant, tag: FmtStr },
    End { time: Instant },
    // TODO: Add Pause, Resume to help with things like the
    // amortized cost of expanding the length of the events
    // array getting reported as a part of another operation.
}

struct SpanGuard;

impl Drop for SpanGuard {
    #[inline(always)]
    fn drop(&mut self) {
        end();
    }
}

#[inline(always)]
#[must_use = "Use a let binding to extend the lifetime of the guard to the scope which to profile."]
pub fn start_guard(name: impl Into<FmtStr>) -> impl Drop {
    start(name);
    SpanGuard
}

#[cfg(feature = "enabled")]
/// Starts profiling a section. Every start MUST be followed by a corresponding
/// call to end()
pub fn start(tag: impl Into<FmtStr>) {
    with_events(|events| {
        let event = Event::Start {
            time: Instant::now(),
            tag: tag.into(),
        };
        events.push(event);
    });
}

#[cfg(not(feature = "enabled"))]
#[inline(always)]
pub fn start(_tag: impl Into<FmtStr>) {}

#[cfg(feature = "enabled")]
/// Finish profiling a section.
pub fn end() {
    with_events(|e| {
        e.push(Event::End {
            time: Instant::now(),
        })
    });
}

#[cfg(not(feature = "enabled"))]
#[inline(always)]
pub fn end() {}

#[cfg(feature = "enabled")]
/// Clears all of the recorded info that firestorm has
/// tracked.
pub fn clear() {
    with_events(|e| e.clear());
}

#[cfg(not(feature = "enabled"))]
#[inline(always)]
pub fn clear() {}

#[cfg(feature = "enabled")]
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
#[cfg(feature = "enabled")]
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

#[inline(always)]
#[cfg(not(feature = "enabled"))]
pub fn to_svg<W: std::io::Write>(_writer: &mut W) -> Result<(), std::convert::Infallible> {
    Ok(())
}

/// Returns whether or not
#[inline(always)]
pub fn enabled() -> bool {
    cfg!(feature = "enabled")
}
