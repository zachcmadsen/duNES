// TODO: Figure out a way to remove the allow(dead_code) attributes in all of
// the files.
#![cfg_attr(test, allow(dead_code))]

use crate::{cpu, Emu};

pub enum EventKind {
    Reset,
    Unreachable,
}

struct Event {
    kind: EventKind,
    tick: u64,
}

pub struct Scheduler {
    events: Vec<Event>,
    ticks: u64,
}

impl Scheduler {
    pub fn new() -> Scheduler {
        Scheduler {
            events: vec![Event {
                kind: EventKind::Unreachable,
                tick: u64::MAX,
            }],
            ticks: 0,
        }
    }
}

pub fn tick(emu: &mut Emu) {
    emu.scheduler.ticks += 1;
}

pub fn queue(emu: &mut Emu, kind: EventKind, offset: u64) {
    let tick = emu.scheduler.ticks + offset;
    for i in 0..emu.scheduler.events.len() {
        if tick <= emu.scheduler.events[i].tick {
            emu.scheduler.events.insert(i, Event { kind, tick });
            return;
        }
    }
}

pub fn handle_events(emu: &mut Emu) {
    while emu.scheduler.events[0].tick <= emu.scheduler.ticks {
        match emu.scheduler.events[0].kind {
            EventKind::Reset => cpu::reset(emu),
            EventKind::Unreachable => unreachable!(),
        }
        emu.scheduler.events.remove(0);
    }
}
