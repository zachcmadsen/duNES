use crate::{cpu, Emu};

pub enum EventKind {
    Reset,
    Unreachable,
}

struct Event {
    kind: EventKind,
    time: u64,
}

pub struct Scheduler {
    events: Vec<Event>,
    time: u64,
}

impl Scheduler {
    pub fn new() -> Scheduler {
        Scheduler {
            events: vec![Event {
                kind: EventKind::Unreachable,
                time: u64::MAX,
            }],
            time: 0,
        }
    }
}

pub fn tick(emu: &mut Emu) {
    emu.scheduler.time += 1;
}

pub fn queue(emu: &mut Emu, kind: EventKind, offset: u64) {
    let time = emu.scheduler.time + offset;
    for i in 0..emu.scheduler.events.len() {
        if time <= emu.scheduler.events[i].time {
            emu.scheduler.events.insert(i, Event { kind, time });
            return;
        }
    }
}

pub fn ready(emu: &mut Emu) -> bool {
    emu.scheduler.time <= emu.scheduler.events[0].time
}

pub fn handle(emu: &mut Emu) {
    match emu.scheduler.events[0].kind {
        EventKind::Reset => cpu::reset(emu),
        EventKind::Unreachable => unreachable!(),
    }
}
