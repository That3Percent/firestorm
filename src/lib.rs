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
use std::time::Instant;

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
fn lines() -> Vec<String> {
    with_events(|events| {
        struct Frame {
            name: String,
            start: Instant,
        }
        let mut stack = Vec::<Frame>::new();
        let mut collapsed = HashMap::<_, u128>::new();

        for event in events.iter() {
            match event {
                Event::Start { time, tag } => {
                    let mut name = format!("{}", tag).replace(";", "").replace(" ", "_");
                    if let Some(parent) = stack.last() {
                        name = format!("{};{}", &parent.name, name);
                    }
                    let frame = Frame {
                        name: name,
                        start: *time,
                    };
                    stack.push(frame);
                }
                Event::End { time } => {
                    let Frame { name, start } = stack.pop().unwrap();
                    let elapsed = (*time - start).as_nanos();
                    let entry = collapsed.entry(name).or_default();
                    *entry = entry.wrapping_add(elapsed);
                    if let Some(parent) = stack.last() {
                        let entry = collapsed.entry(parent.name.clone()).or_default();
                        *entry = entry.wrapping_sub(elapsed);
                    }
                }
            }
        }
        assert!(stack.is_empty(), "Mimatched start/end");

        //let mut collapsed: Vec<_> = collapsed.iter().collect();
        //collapsed.sort_by_key(|(_, time)| *time);

        collapsed
            .iter()
            .map(|(name, time)| format!("{} {}", name, time))
            .collect()
    })
}

/// Write the data to an svg
#[cfg(feature = "enabled")]
pub fn to_svg<W: std::io::Write, F: FnOnce() -> W>(f: F) -> Result<(), impl std::error::Error> {
    let lines = lines();
    let mut options = flamegraph::Options {
        count_name: "ns".to_owned(),
        hash: true,
        ..Default::default()
    };
    flamegraph::from_lines(&mut options, lines.iter().rev().map(|s| s.as_str()), f())
}

#[cfg(not(feature = "enabled"))]
pub fn to_svg<W: std::io::Write, F: FnOnce() -> W>(_f: F) -> Result<(), std::convert::Infallible> {
    Ok(())
}
