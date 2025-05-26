//! # midi_convert
//!
//! Utilities for converting to/from `midi_types::MidiMessage`
//!
//! # Examples
//!
//! Render a `MidiMessage` into a byte slice.
//! ```
//! use midi_convert::render_slice::MidiRenderSlice;
//! use midi_types::MidiMessage;
//!
//! let mut s = [0u8; 3];
//! let m = MidiMessage::NoteOn(2.into(), 0x76.into(), 0x34.into());
//! assert_eq!(m.render_slice(&mut s), 3);
//! assert_eq!(s, [0x92, 0x76, 0x34]);
//! ```
//!
//! Try to extract a `MidiMessage` from a byte slice.
//! ```
//! use midi_convert::parse::{MidiTryParseSlice, MidiParseError};
//! use midi_types::MidiMessage;
//!
//! assert_eq!(MidiMessage::try_parse_slice(&[0x92, 0x76, 0x34]), Ok(MidiMessage::NoteOn(2.into(), 0x76.into(), 0x34.into())));
//! assert_eq!(MidiMessage::try_parse_slice(&[0x92]), Err(MidiParseError::BufferTooShort));
//! ```
//!
//! Parse a byte stream, returning `MidiMessage` found along the way.
//! ```
//! use midi_convert::parse::{MidiParser};
//! use midi_types::MidiMessage;
//!
//! let mut parser = MidiParser::new();
//! assert_eq!(parser.parse(0x92), None);
//! assert_eq!(parser.parse(0x76), None);
//! assert_eq!(parser.parse(0x34), Some(MidiMessage::NoteOn(2.into(), 0x76.into(), 0x34.into())));
//! ```
//!

#![no_std]
#[warn(missing_debug_implementations, missing_docs)]
pub mod parse;
pub mod render;
pub mod render_slice;

pub use midi_types;

#[cfg(test)]
pub(crate) mod test {
    use {
        crate::{parse::MidiTryParseSlice, render_slice::MidiRenderSlice},
        midi_types::{Channel, Control, MidiMessage, Note, Program, QuarterFrame, Value7, Value14},
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
        let mut buf3 = [0, 0, 0];
        let mut buf100 = [0; 100];
        for v in (*TEST_1BYTE).iter() {
            assert_eq!(1, v.render_slice(&mut buf3), "{:?}", v);
            assert_eq!(Ok(*v), MidiMessage::try_parse_slice(buf3.as_slice()));
            assert_eq!(1, v.render_slice(&mut buf100), "{:?}", v);
            assert_eq!(
                Ok(*v),
                MidiMessage::try_parse_slice(buf100.as_slice())
            );
        }

        for v in (*TEST_2BYTE).iter() {
            assert_eq!(2, v.render_slice(&mut buf3), "{:?}", v);
            assert_eq!(Ok(*v), MidiMessage::try_parse_slice(buf3.as_slice()));
            assert_eq!(2, v.render_slice(&mut buf100), "{:?}", v);
            assert_eq!(
                Ok(*v),
                MidiMessage::try_parse_slice(buf100.as_slice())
            );
        }

        for v in (*TEST_3BYTE).iter() {
            assert_eq!(3, v.render_slice(&mut buf3), "{:?}", v);
            assert_eq!(Ok(*v), MidiMessage::try_parse_slice(buf3.as_slice()));
            assert_eq!(3, v.render_slice(&mut buf100), "{:?}", v);
            assert_eq!(
                Ok(*v),
                MidiMessage::try_parse_slice(buf100.as_slice())
            );
        }
    }
}
