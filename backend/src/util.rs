macro_rules! word {
    ($low:expr, $high:expr) => {
        ($low as u16) | (($high as u16) << 8)
    };
}

pub(crate) use word;
