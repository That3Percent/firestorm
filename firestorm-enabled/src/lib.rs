// The backwards compatability policy:
// When core needs to change:
//    It can release a new version
//    And every point release of previous APIs can be upgraded to use it.
//    So long as exact versions are never relied upon this should work.
//
// When a new version comes out, it needs to enable the previous version
//    (which happens recursively)
//    It would be good if this only brought in the profiling part and not
//    the drawing part, but I can fix that later.

use std::{error::Error, fs::create_dir_all, io::Write, path::Path};

extern crate inferno;

use {firestorm_core::*, inferno::flamegraph, std::collections::HashMap};
pub mod internal;

#[macro_export]
macro_rules! profile_fn {
    ($($t:tt)*) => {
        let _firestorm_fn_guard = {
            let event_data = $crate::internal::EventData::Start(
                $crate::internal::Start::Func {
                    signature: &stringify!($($t)*),
                }
            );
            $crate::internal::start(event_data);
            $crate::internal::SpanGuard
        };
    };
}

#[macro_export]
macro_rules! profile_method {
    ($($t:tt)*) => {
        let _firestorm_method_guard = {
            let event_data = $crate::internal::EventData::Start(
                $crate::internal::Start::Method {
                    signature: &stringify!($($t)*),
                    typ: ::std::any::type_name::<Self>(),
                }
            );
            $crate::internal::start(event_data);
            $crate::internal::SpanGuard
        };
    };
}

#[macro_export]
macro_rules! profile_section {
    ($name:ident) => {
        #[allow(unused_variables)]
        let $name = {
            let event_data = $crate::internal::EventData::Start($crate::internal::Start::Section {
                name: &stringify!($name),
            });
            $crate::internal::start(event_data);
            $crate::internal::SpanGuard
        };
    };
}

/// Clears all of the recorded info that firestorm has tracked in this thread.
pub fn clear() {
    with_events(|e| e.clear());
}

fn inferno_valid_chars(s: &str) -> String {
    s.replace(";", "").replace(" ", "")
}

fn format_start(tag: &Start) -> String {
    let mut s = String::new();
    match tag {
        Start::Method { typ, signature } => {
            s += typ;
            s += "::";
            s += signature;
        }
        Start::Func { signature } => {
            s += signature;
        }
        Start::Section { name } => {
            s += name;
        }
        _ => s += "Unsupported",
    }
    s
}

/// Convert events to the format that inferno is expecting
fn lines(mode: Mode) -> Vec<String> {
    with_events(|events| {
        struct Frame {
            name: String,
            start: TimeSample,
        }
        struct Line {
            name: String,
            duration: u64,
        }

        let mut stack = Vec::<Frame>::new();
        let mut collapsed = HashMap::<_, u64>::new();
        let mut lines = Vec::<Line>::new();

        for event in events.iter() {
            let time = event.time;
            match &event.data {
                EventData::Start(tag) => {
                    let mut s = format_start(tag);
                    s = inferno_valid_chars(&s);
                    if let Some(parent) = stack.last() {
                        if !matches!(mode, Mode::OwnTime) {
                            s = format!("{};{}", &parent.name, s);
                        }
                        if mode == Mode::TimeAxis {
                            lines.push(Line {
                                name: parent.name.clone(),
                                duration: time - parent.start,
                            });
                        }
                    }
                    let frame = Frame {
                        name: s,
                        start: time,
                    };
                    stack.push(frame);
                }
                EventData::End => {
                    let Frame { name, start } = stack.pop().unwrap();
                    let elapsed = time - start;
                    match mode {
                        Mode::Merged | Mode::OwnTime => {
                            let entry = collapsed.entry(name).or_default();
                            *entry = entry.wrapping_add(elapsed);
                            if let Some(parent) = stack.last() {
                                let entry = collapsed.entry(parent.name.clone()).or_default();
                                *entry = entry.wrapping_sub(elapsed);
                            }
                        }
                        Mode::TimeAxis => {
                            lines.push(Line {
                                name,
                                duration: elapsed,
                            });
                            if let Some(parent) = stack.last_mut() {
                                parent.start = time;
                            }
                        }
                    }
                }
                _ => panic!("Unsupported event data. Update Firestorm."),
            }
        }
        assert!(stack.is_empty(), "Mimatched start/end");

        fn format_line(name: &str, duration: &u64) -> Option<String> {
            if *duration == 0 {
                None
            } else {
                Some(format!("{} {}", name, duration))
            }
        }

        match mode {
            Mode::Merged => collapsed
                .iter()
                .filter_map(|(name, duration)| format_line(name, duration))
                .collect(),
            Mode::TimeAxis => lines
                .iter()
                .filter_map(|Line { name, duration }| format_line(name, duration))
                .collect(),
            Mode::OwnTime => {
                let mut collapsed: Vec<_> = collapsed
                    .into_iter()
                    .filter(|(_, duration)| *duration != 0)
                    .collect();
                collapsed.sort_by_key(|(_, duration)| u64::MAX - *duration);

                for i in 1..collapsed.len() {
                    collapsed[i - 1].1 -= collapsed[i].1;
                }

                let mut collapsed = collapsed.into_iter();
                let mut lines = Vec::new();
                if let Some(item) = collapsed.next() {
                    let mut name = item.0.clone();
                    lines.push(format_line(&name, &item.1).unwrap());
                    for item in collapsed {
                        name = format!("{};{}", name, &item.0);
                        lines.push(format_line(&name, &item.1).unwrap());
                    }
                }
                lines
            }
        }
    })
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum Mode {
    TimeAxis,
    /// Merges all instances of the same stack into a single bar.
    /// This may give a better overview of, for example, how much total
    /// time a method took. But, will not retain information like how
    /// many times a method was called.
    Merged,
    /// The stacks have nothing to do with callstacks, this is just a
    /// bar graph on it's side.
    OwnTime,
}

/// Save the flamegraph to a folder.
pub fn save<P: AsRef<Path>>(path: P) -> Result<(), Box<dyn Error>> {
    let data_dir = path.as_ref().join("firestorm");
    create_dir_all(&data_dir)?;
    for (mode, name) in [
        (Mode::OwnTime, "owntime"),
        (Mode::TimeAxis, "timeaxis"),
        (Mode::Merged, "merged"),
    ]
    .iter()
    {
        let lines = lines(*mode);

        /*
        // Output lines for debugging
        use std::io::Write;
        let mut f = std::fs::File::create("C:\\git\\flames.txt")?;
        for line in lines.iter() {
            f.write(line.as_bytes())?;
            f.write("\n".as_bytes())?;
        }
        drop(f);
        */

        let mut fg_opts = flamegraph::Options::default();
        fg_opts.count_name = "".to_owned();
        fg_opts.title = "".to_owned();
        fg_opts.hash = true;
        fg_opts.flame_chart = matches!(mode, Mode::TimeAxis);
        let name = data_dir.join(name.to_string() + ".svg");
        let mut writer = std::fs::File::create(name)?;
        flamegraph::from_lines(
            &mut fg_opts,
            lines.iter().rev().map(|s| s.as_str()),
            &mut writer,
        )?;
    }

    let name = path.as_ref().join("firestorm.html");
    let mut writer = std::fs::File::create(name)?;
    let html = include_bytes!("firestorm.html");
    writer.write_all(html)?;

    Ok(())
}

/// Finish profiling a section.
pub(crate) fn end() {
    with_events(|events| {
        events.push(Event {
            time: TimeSample::now(),
            data: EventData::End,
        })
    });
}

/// Unsafe! This MUST not be used recursively
/// TODO: Verify in Debug this is not used recursively
pub(crate) fn with_events<T>(f: impl FnOnce(&mut Vec<Event>) -> T) -> T {
    EVENTS.with(|e| {
        let r = unsafe { &mut *e.get() };
        f(r)
    })
}

/// Returns whether or not firestorm is enabled
#[inline(always)]
pub const fn enabled() -> bool {
    true
}

pub fn bench<F: Fn(), P: AsRef<Path>>(path: P, f: F) -> Result<(), Box<dyn Error>> {
    // Warmup - pre-allocates memory for samples
    f();
    clear();

    f();
    save(path)
}
