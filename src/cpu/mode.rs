macro_rules! absolute {
    ($cpu:ident) => {
        $cpu.pins.address = $cpu.consume_word();
    };
}

macro_rules! absolute_x_read {
    ($cpu:ident) => {
        let (low, page_cross) = $cpu.consume_byte().overflowing_add($cpu.x);
        let high = $cpu.consume_byte();

        let effective_address =
            (high.wrapping_add(page_cross as u8) as u16) << 8 | (low as u16);

        // If the effective address is invalid, i.e., it crossed a
        // page, then it takes an extra read cycle to fix it. Write
        // instructions always have the extra read since they can't
        // undo a write to an invalid address.
        if page_cross {
            $cpu.read_byte((high as u16) << 8 | low as u16);
        }

        $cpu.pins.address = effective_address;
    };
}

macro_rules! absolute_x_write {
    ($cpu:ident) => {
        let (low, page_cross) = $cpu.consume_byte().overflowing_add($cpu.x);
        let high = $cpu.consume_byte();
        let effective_address =
            (high.wrapping_add(page_cross as u8) as u16) << 8 | (low as u16);
        $cpu.read_byte((high as u16) << 8 | low as u16);

        $cpu.pins.address = effective_address;
    };
}

macro_rules! absolute_y_read {
    ($cpu:ident) => {
        let (low, page_cross) = $cpu.consume_byte().overflowing_add($cpu.y);
        let high = $cpu.consume_byte();

        let effective_address =
            (high.wrapping_add(page_cross as u8) as u16) << 8 | (low as u16);

        // If the effective address is invalid, i.e., it crossed a
        // page, then it takes an extra read cycle to fix it. Write
        // instructions always have the extra read since they can't
        // undo a write to an invalid address.
        if page_cross {
            $cpu.read_byte((high as u16) << 8 | low as u16);
        }

        $cpu.pins.address = effective_address;
    };
}

macro_rules! absolute_y_write {
    ($cpu:ident) => {
        let (low, page_cross) = $cpu.consume_byte().overflowing_add($cpu.y);
        let high = $cpu.consume_byte();
        let effective_address =
            (high.wrapping_add(page_cross as u8) as u16) << 8 | (low as u16);
        $cpu.read_byte((high as u16) << 8 | low as u16);

        $cpu.pins.address = effective_address;
    };
}

macro_rules! immediate {
    ($cpu:ident) => {
        $cpu.pins.address = $cpu.pc;
        $cpu.pc = $cpu.pc.wrapping_add(1);
    };
}

macro_rules! indexed_indirect {
    ($cpu:ident) => {
        let ptr = $cpu.consume_byte();
        $cpu.read_byte(ptr as u16);
        $cpu.pins.address =
            $cpu.read_word_bugged(ptr.wrapping_add($cpu.x) as u16);
    };
}

macro_rules! indirect_indexed_read {
    ($cpu:ident) => {
        let ptr = $cpu.consume_byte();

        let (low, did_cross_page) =
            $cpu.read_byte(ptr as u16).overflowing_add($cpu.y);
        let high = $cpu.read_byte(ptr.wrapping_add(1) as u16);

        let effective_address =
            (high.wrapping_add(did_cross_page as u8) as u16) << 8
                | (low as u16);

        // If the effective address is invalid, i.e., it crossed a
        // page, then it takes an extra read cycle to fix it. Write
        // instructions always have the extra read since they can't
        // undo a write to an invalid address.
        if did_cross_page {
            $cpu.read_byte((high as u16) << 8 | low as u16);
        }

        $cpu.pins.address = effective_address
    };
}

macro_rules! indirect_indexed_write {
    ($cpu:ident) => {
        let ptr = $cpu.consume_byte();

        let (low, did_cross_page) =
            $cpu.read_byte(ptr as u16).overflowing_add($cpu.y);
        let high = $cpu.read_byte(ptr.wrapping_add(1) as u16);

        let effective_address =
            (high.wrapping_add(did_cross_page as u8) as u16) << 8
                | (low as u16);

        $cpu.read_byte((high as u16) << 8 | low as u16);

        $cpu.pins.address = effective_address
    };
}

macro_rules! indirect {
    ($cpu:ident) => {
        let ptr = $cpu.consume_word();
        $cpu.pins.address = $cpu.read_word_bugged(ptr);
    };
}

macro_rules! zero_page {
    ($cpu:ident) => {
        $cpu.pins.address = $cpu.consume_byte() as u16;
    };
}

macro_rules! zero_page_x {
    ($cpu:ident) => {
        let address = $cpu.consume_byte();
        $cpu.read_byte(address as u16);
        $cpu.pins.address = address.wrapping_add($cpu.x) as u16
    };
}

macro_rules! zero_page_y {
    ($cpu:ident) => {
        let address = $cpu.consume_byte();
        $cpu.read_byte(address as u16);
        $cpu.pins.address = address.wrapping_add($cpu.y) as u16
    };
}

pub(crate) use absolute;
pub(crate) use absolute_x_read;
pub(crate) use absolute_x_write;
pub(crate) use absolute_y_read;
pub(crate) use absolute_y_write;
pub(crate) use immediate;
pub(crate) use indexed_indirect;
pub(crate) use indirect;
pub(crate) use indirect_indexed_read;
pub(crate) use indirect_indexed_write;
pub(crate) use zero_page;
pub(crate) use zero_page_x;
pub(crate) use zero_page_y;
