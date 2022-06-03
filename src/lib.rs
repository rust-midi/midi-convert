#![no_std]
#[warn(missing_debug_implementations, missing_docs)]
pub mod parse;
pub mod render;

pub use midi_types;

#[cfg(test)]
pub(crate) mod test {
    use {
        crate::{parse::parse_slice, render::render},
        midi_types::{Channel, Control, MidiMessage, Note, Program, QuarterFrame, Value14, Value7},
    };

    lazy_static::lazy_static! {
        pub(crate) static ref TEST_1BYTE: [MidiMessage; 7] = [
            MidiMessage::TuneRequest,
            MidiMessage::TimingClock,
            MidiMessage::Start,
            MidiMessage::Continue,
            MidiMessage::Stop,
            MidiMessage::ActiveSensing,
            MidiMessage::Reset,
        ];
        pub(crate) static ref TEST_2BYTE: [MidiMessage; 4] = [
            MidiMessage::ProgramChange(Channel::from(0), Program::from(0)),
            MidiMessage::ChannelPressure(Channel::from(1), Value7::from(2)),
            MidiMessage::QuarterFrame(QuarterFrame::from(23)),
            MidiMessage::SongSelect(Value7::from(3)),
        ];
        pub(crate) static ref TEST_3BYTE: [MidiMessage; 6] = [
            MidiMessage::NoteOff(Channel::from(2), Note::from(3), Value7::from(1)),
            MidiMessage::NoteOn(Channel::from(3), Note::from(120), Value7::from(120)),
            MidiMessage::KeyPressure(Channel::from(3), Note::from(120), Value7::from(1)),
            MidiMessage::ControlChange(Channel::from(5), Control::from(23), Value7::from(23)),
            MidiMessage::PitchBendChange(Channel::from(15), Value14::from((23, 23))),
            MidiMessage::SongPositionPointer(Value14::from((0, 0))),
        ];
    }

    #[test]
    fn parse_rendered() {
        let mut buf1 = [0];
        let mut buf2 = [0, 0];
        let mut buf3 = [0, 0, 0];
        let mut buf100 = [0; 100];
        for v in (*TEST_1BYTE).iter() {
            assert_eq!(Ok(1), render(&v, &mut buf1), "{:?}", v);
            assert_eq!(Ok(v.clone()), parse_slice(buf1.as_slice()));
            assert_eq!(Ok(1), render(&v, &mut buf2), "{:?}", v);
            assert_eq!(Ok(v.clone()), parse_slice(buf2.as_slice()));
            assert_eq!(Ok(1), render(&v, &mut buf100), "{:?}", v);
            assert_eq!(Ok(v.clone()), parse_slice(buf100.as_slice()));
        }

        for v in (*TEST_2BYTE).iter() {
            assert_eq!(Ok(2), render(&v, &mut buf2), "{:?}", v);
            assert_eq!(Ok(v.clone()), parse_slice(buf2.as_slice()));
            assert_eq!(Ok(2), render(&v, &mut buf100), "{:?}", v);
            assert_eq!(Ok(v.clone()), parse_slice(buf100.as_slice()));
        }

        for v in (*TEST_3BYTE).iter() {
            assert_eq!(Ok(3), render(&v, &mut buf3), "{:?}", v);
            assert_eq!(Ok(v.clone()), parse_slice(buf3.as_slice()));
            assert_eq!(Ok(3), render(&v, &mut buf100), "{:?}", v);
            assert_eq!(Ok(v.clone()), parse_slice(buf100.as_slice()));
        }
    }
}
