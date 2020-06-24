use {firestorm_core::*, inferno::flamegraph, std::collections::HashMap};

#[macro_export]
macro_rules! profile_fn {
    ($($t:tt)*) => {
        let _firestorm_fn_guard = {
            let event_data = &$crate::internal::EventData::Start(
                $crate::internal::Start::Func {
                    signature: stringify!($($t)*),
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
            const FIRESTORM_STRUCT_NAME: &'static str = ::std::any::type_name::<Self>();
            let event_data: &'static _ = &$crate::internal::EventData::Start(
                $crate::internal::Start::Method {
                    signature: stringify!($($t)*),
                    typ: FIRESTORM_STRUCT_NAME,
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
            let event_data =
                &$crate::internal::EventData::Start($crate::internal::Start::Section {
                    name: stringify!($name),
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

/// Convert events to the format that inferno is expecting
fn lines(options: &Options) -> Vec<String> {
    with_events(|events| {
        struct Frame {
            name: String,
            start: TimeSample,
        }
        struct Line {
            name: String,
            duration: u64,
        }

        fn push_line(lines: &mut Vec<Line>, name: String, duration: u64) {
            if let Some(prev) = lines.last_mut() {
                if &prev.name == &name {
                    prev.duration += duration;
                    return;
                }
            }
            lines.push(Line { name, duration });
        }

        let mut stack = Vec::<Frame>::new();
        let mut collapsed = HashMap::<_, u64>::new();
        let mut lines = Vec::<Line>::new();

        for event in events.iter() {
            let time = event.time;
            match event.data {
                EventData::Start(tag) => {
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
                    // Characters which are not supported by inferno
                    s = s.replace(";", "").replace(" ", "");
                    if let Some(parent) = stack.last() {
                        s = format!("{};{}", &parent.name, s);
                        if !options.merge {
                            push_line(&mut lines, s.clone(), time - parent.start);
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
                    if options.merge {
                        let entry = collapsed.entry(name).or_default();
                        *entry = entry.wrapping_add(elapsed);
                        if let Some(parent) = stack.last() {
                            let entry = collapsed.entry(parent.name.clone()).or_default();
                            *entry = entry.wrapping_sub(elapsed);
                        }
                    } else {
                        push_line(&mut lines, name, elapsed);
                        if let Some(parent) = stack.last_mut() {
                            parent.start = time;
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

        if options.merge {
            collapsed
                .iter()
                .filter_map(|(name, duration)| format_line(name, duration))
                .collect()
        } else {
            lines
                .iter()
                .filter_map(|Line { name, duration }| format_line(name, duration))
                .collect()
        }
    })
}

/// This API is unstable, and will likely go away
/// Ultimately it would be best to output a webpage with
/// both the merged/unmerged outputs to show
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
    with_events(|events| {
        events.push(Event {
            time: TimeSample::now(),
            data: &EventData::End,
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
