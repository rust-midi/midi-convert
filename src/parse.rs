//! Parse midi messages
use midi_types::{
    Channel, Control, MidiMessage, Note, Program, QuarterFrame, Value7, Value14, status::*,
};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
/// Errors parsing.
pub enum MidiParseError {
    /// Input buffer wasn't long enough to parse anything
    BufferTooShort,

    /// Couldn't find a valid message
    MessageNotFound,
}

/// Trait for parsing a byte slice into a MidiMessage
pub trait MidiTryParseSlice: Sized {
    /// try to parse
    fn try_parse_slice(buf: &[u8]) -> Result<Self, MidiParseError>;
}

/// A parser that parses a byte at a time.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
pub struct MidiParser {
    state: MidiParserState,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
enum MidiParserState {
    Idle,
    NoteOnRecvd(Channel),
    NoteOnNoteRecvd(Channel, Note),

    NoteOffRecvd(Channel),
    NoteOffNoteRecvd(Channel, Note),

    KeyPressureRecvd(Channel),
    KeyPressureNoteRecvd(Channel, Note),

    ControlChangeRecvd(Channel),
    ControlChangeControlRecvd(Channel, Control),

    ProgramChangeRecvd(Channel),

    ChannelPressureRecvd(Channel),

    PitchBendRecvd(Channel),
    PitchBendLsbRecvd(Channel, u8),

    QuarterFrameRecvd,

    SongPositionRecvd,
    SongPositionLsbRecvd(u8),

    SongSelectRecvd,
}

/// Check if most significant bit is set which signifies a Midi status byte
fn is_status_byte(byte: u8) -> bool {
    byte & 0x80 == 0x80
}

/// Check if a byte corresponds to 0x1111xxxx which signifies either a system common or realtime message
fn is_system_message(byte: u8) -> bool {
    byte & 0xf0 == 0xf0
}

/// Split the message and channel part of a channel voice message
fn split_message_and_channel(byte: u8) -> (u8, Channel) {
    (byte & 0xf0u8, (byte & 0x0fu8).into())
}

/// Parse Midi messages byte at a time.
///
/// Returns parsed Midi messages whenever one is completed.
impl MidiParser {
    /// Initialize midiparser state
    pub fn new() -> Self {
        Self::default()
    }

    /// Parse midi event byte by byte. Call this whenever a byte is received. When a midi-event is
    /// completed it is returned, otherwise this method updates the internal midiparser state and
    /// and returns none.
    pub fn parse(&mut self, byte: u8) -> Option<MidiMessage> {
        if is_status_byte(byte) {
            if is_system_message(byte) {
                match byte {
                    // System common messages, these should reset parsing other messages
                    0xf0 => {
                        // System exclusive
                        self.state = MidiParserState::Idle;
                        None
                    }
                    0xf1 => {
                        // Midi time code quarter frame
                        self.state = MidiParserState::QuarterFrameRecvd;
                        None
                    }
                    0xf2 => {
                        // Song position pointer
                        self.state = MidiParserState::SongPositionRecvd;
                        None
                    }
                    0xf3 => {
                        // Song select
                        self.state = MidiParserState::SongSelectRecvd;
                        None
                    }
                    0xf6 => {
                        // Tune request
                        self.state = MidiParserState::Idle;
                        Some(MidiMessage::TuneRequest)
                    }
                    0xf7 => {
                        // End of exclusive
                        self.state = MidiParserState::Idle;
                        None
                        // Some(MidiMessage::EndOfExclusive)
                    }

                    // System realtime messages
                    0xf8 => Some(MidiMessage::TimingClock),
                    0xf9 => None, // Reserved
                    0xfa => Some(MidiMessage::Start),
                    0xfb => Some(MidiMessage::Continue),
                    0xfc => Some(MidiMessage::Stop),
                    0xfd => None, // Reserved
                    0xfe => Some(MidiMessage::ActiveSensing),
                    0xff => Some(MidiMessage::Reset),

                    _ => {
                        // Undefined messages like 0xf4 and should end up here
                        self.state = MidiParserState::Idle;
                        None
                    }
                }
            } else {
                // Channel voice message

                let (message, channel) = split_message_and_channel(byte);

                match message {
                    0x80 => {
                        self.state = MidiParserState::NoteOffRecvd(channel);
                        None
                    }
                    0x90 => {
                        self.state = MidiParserState::NoteOnRecvd(channel);
                        None
                    }
                    0xA0 => {
                        self.state = MidiParserState::KeyPressureRecvd(channel);
                        None
                    }
                    0xB0 => {
                        self.state = MidiParserState::ControlChangeRecvd(channel);
                        None
                    }
                    0xC0 => {
                        self.state = MidiParserState::ProgramChangeRecvd(channel);
                        None
                    }
                    0xD0 => {
                        self.state = MidiParserState::ChannelPressureRecvd(channel);
                        None
                    }
                    0xE0 => {
                        self.state = MidiParserState::PitchBendRecvd(channel);
                        None
                    }
                    _ => None,
                }
            }
        } else {
            match self.state {
                MidiParserState::NoteOffRecvd(channel) => {
                    self.state = MidiParserState::NoteOffNoteRecvd(channel, byte.into());
                    None
                }
                MidiParserState::NoteOffNoteRecvd(channel, note) => {
                    self.state = MidiParserState::NoteOffRecvd(channel);
                    Some(MidiMessage::NoteOff(channel, note, byte.into()))
                }

                MidiParserState::NoteOnRecvd(channel) => {
                    self.state = MidiParserState::NoteOnNoteRecvd(channel, byte.into());
                    None
                }
                MidiParserState::NoteOnNoteRecvd(channel, note) => {
                    self.state = MidiParserState::NoteOnRecvd(channel);
                    Some(MidiMessage::NoteOn(channel, note, byte.into()))
                }

                MidiParserState::KeyPressureRecvd(channel) => {
                    self.state = MidiParserState::KeyPressureNoteRecvd(channel, byte.into());
                    None
                }
                MidiParserState::KeyPressureNoteRecvd(channel, note) => {
                    self.state = MidiParserState::KeyPressureRecvd(channel);
                    Some(MidiMessage::KeyPressure(channel, note, byte.into()))
                }

                MidiParserState::ControlChangeRecvd(channel) => {
                    self.state = MidiParserState::ControlChangeControlRecvd(channel, byte.into());
                    None
                }
                MidiParserState::ControlChangeControlRecvd(channel, control) => {
                    self.state = MidiParserState::ControlChangeRecvd(channel);
                    Some(MidiMessage::ControlChange(channel, control, byte.into()))
                }

                MidiParserState::ProgramChangeRecvd(channel) => {
                    Some(MidiMessage::ProgramChange(channel, byte.into()))
                }

                MidiParserState::ChannelPressureRecvd(channel) => {
                    Some(MidiMessage::ChannelPressure(channel, byte.into()))
                }

                MidiParserState::PitchBendRecvd(channel) => {
                    self.state = MidiParserState::PitchBendLsbRecvd(channel, byte);
                    None
                }
                MidiParserState::PitchBendLsbRecvd(channel, lsb) => {
                    self.state = MidiParserState::PitchBendRecvd(channel);
                    Some(MidiMessage::PitchBendChange(channel, (byte, lsb).into()))
                }
                MidiParserState::QuarterFrameRecvd => Some(MidiMessage::QuarterFrame(byte.into())),
                MidiParserState::SongPositionRecvd => {
                    self.state = MidiParserState::SongPositionLsbRecvd(byte);
                    None
                }
                MidiParserState::SongPositionLsbRecvd(lsb) => {
                    self.state = MidiParserState::SongPositionRecvd;
                    Some(MidiMessage::SongPositionPointer((byte, lsb).into()))
                }
                MidiParserState::SongSelectRecvd => Some(MidiMessage::SongSelect(byte.into())),
                _ => None,
            }
        }
    }
}

impl Default for MidiParser {
    fn default() -> Self {
        MidiParser {
            state: MidiParserState::Idle,
        }
    }
}

const NOTE_OFF_END: u8 = NOTE_OFF + 0x0F;
const NOTE_ON_END: u8 = NOTE_ON + 0x0F;
const KEY_PRESSURE_END: u8 = KEY_PRESSURE + 0x0F;
const CONTROL_CHANGE_END: u8 = CONTROL_CHANGE + 0x0F;
const PITCH_BEND_CHANGE_END: u8 = PITCH_BEND_CHANGE + 0x0F;
const PROGRAM_CHANGE_END: u8 = PROGRAM_CHANGE + 0x0F;
const CHANNEL_PRESSURE_END: u8 = CHANNEL_PRESSURE + 0x0F;

//parse helper guard
fn check_len<F: Fn() -> Result<MidiMessage, MidiParseError>>(
    buf: &[u8],
    len: usize,
    func: F,
) -> Result<MidiMessage, MidiParseError> {
    if buf.len() >= len {
        func()
    } else {
        Err(MidiParseError::BufferTooShort)
    }
}

/// Parse a byte slice for a MidiMessage
impl MidiTryParseSlice for MidiMessage {
    fn try_parse_slice(buf: &[u8]) -> Result<MidiMessage, MidiParseError> {
        if buf.is_empty() {
            Err(MidiParseError::BufferTooShort)
        } else {
            let chan = |status: u8| -> Channel { Channel::from(status & 0x0F) };
            match buf[0] {
                //1 byte
                TUNE_REQUEST => Ok(MidiMessage::TuneRequest),
                TIMING_CLOCK => Ok(MidiMessage::TimingClock),
                START => Ok(MidiMessage::Start),
                CONTINUE => Ok(MidiMessage::Continue),
                STOP => Ok(MidiMessage::Stop),
                ACTIVE_SENSING => Ok(MidiMessage::ActiveSensing),
                RESET => Ok(MidiMessage::Reset),

                //2 byte
                s @ PROGRAM_CHANGE..=PROGRAM_CHANGE_END => check_len(buf, 2, || {
                    Ok(MidiMessage::ProgramChange(chan(s), Program::from(buf[1])))
                }),
                s @ CHANNEL_PRESSURE..=CHANNEL_PRESSURE_END => check_len(buf, 2, || {
                    Ok(MidiMessage::ChannelPressure(chan(s), Value7::from(buf[1])))
                }),
                QUARTER_FRAME => check_len(buf, 2, || {
                    Ok(MidiMessage::QuarterFrame(QuarterFrame::from(buf[1])))
                }),
                SONG_SELECT => {
                    check_len(buf, 2, || Ok(MidiMessage::SongSelect(Value7::from(buf[1]))))
                }

                //3 byte
                s @ NOTE_OFF..=NOTE_OFF_END => check_len(buf, 3, || {
                    Ok(MidiMessage::NoteOff(
                        chan(s),
                        Note::from(buf[1]),
                        Value7::from(buf[2]),
                    ))
                }),
                s @ NOTE_ON..=NOTE_ON_END => check_len(buf, 3, || {
                    Ok(MidiMessage::NoteOn(
                        chan(s),
                        Note::from(buf[1]),
                        Value7::from(buf[2]),
                    ))
                }),
                s @ KEY_PRESSURE..=KEY_PRESSURE_END => check_len(buf, 3, || {
                    Ok(MidiMessage::KeyPressure(
                        chan(s),
                        Note::from(buf[1]),
                        Value7::from(buf[2]),
                    ))
                }),
                s @ CONTROL_CHANGE..=CONTROL_CHANGE_END => check_len(buf, 3, || {
                    Ok(MidiMessage::ControlChange(
                        chan(s),
                        Control::from(buf[1]),
                        Value7::from(buf[2]),
                    ))
                }),
                s @ PITCH_BEND_CHANGE..=PITCH_BEND_CHANGE_END => check_len(buf, 3, || {
                    Ok(MidiMessage::PitchBendChange(
                        chan(s),
                        Value14::from((buf[1], buf[2])),
                    ))
                }),
                SONG_POSITION_POINTER => check_len(buf, 3, || {
                    Ok(MidiMessage::SongPositionPointer(Value14::from((
                        buf[1], buf[2],
                    ))))
                }),

                _ => Err(MidiParseError::MessageNotFound),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate std;
    use super::*;
    use std::vec::Vec;

    #[test]
    fn should_parse_status_byte() {
        assert!(is_status_byte(0x80u8));
        assert!(is_status_byte(0x94u8));
        assert!(!is_status_byte(0x00u8));
        assert!(!is_status_byte(0x78u8));
    }

    #[test]
    fn should_parse_system_message() {
        assert!(is_system_message(0xf0));
        assert!(is_system_message(0xf4));
        assert!(!is_system_message(0x0f));
        assert!(!is_system_message(0x77));
    }

    #[test]
    fn should_split_message_and_channel() {
        let (message, channel) = split_message_and_channel(0x91u8);
        assert_eq!(message, 0x90u8);
        assert_eq!(channel, 1.into());
    }

    #[test]
    fn should_parse_note_off() {
        MidiParser::new().assert_result(
            &[0x82, 0x76, 0x34],
            &[MidiMessage::NoteOff(2.into(), 0x76.into(), 0x34.into())],
        );
    }

    #[test]
    fn should_handle_note_off_running_state() {
        MidiParser::new().assert_result(
            &[
                0x82, 0x76, 0x34, // First note_off
                0x33, 0x65, // Second note_off without status byte
            ],
            &[
                MidiMessage::NoteOff(2.into(), 0x76.into(), 0x34.into()),
                MidiMessage::NoteOff(2.into(), 0x33.into(), 0x65.into()),
            ],
        );
    }

    #[test]
    fn should_parse_note_on() {
        MidiParser::new().assert_result(
            &[0x91, 0x04, 0x34],
            &[MidiMessage::NoteOn(1.into(), 4.into(), 0x34.into())],
        );
    }

    #[test]
    fn should_handle_note_on_running_state() {
        MidiParser::new().assert_result(
            &[
                0x92, 0x76, 0x34, // First note_on
                0x33, 0x65, // Second note on without status byte
            ],
            &[
                MidiMessage::NoteOn(2.into(), 0x76.into(), 0x34.into()),
                MidiMessage::NoteOn(2.into(), 0x33.into(), 0x65.into()),
            ],
        );
    }

    #[test]
    fn should_parse_keypressure() {
        MidiParser::new().assert_result(
            &[0xAA, 0x13, 0x34],
            &[MidiMessage::KeyPressure(
                10.into(),
                0x13.into(),
                0x34.into(),
            )],
        );
    }

    #[test]
    fn should_handle_keypressure_running_state() {
        MidiParser::new().assert_result(
            &[
                0xA8, 0x77, 0x03, // First key_pressure
                0x14, 0x56, // Second key_pressure without status byte
            ],
            &[
                MidiMessage::KeyPressure(8.into(), 0x77.into(), 0x03.into()),
                MidiMessage::KeyPressure(8.into(), 0x14.into(), 0x56.into()),
            ],
        );
    }

    #[test]
    fn should_parse_control_change() {
        MidiParser::new().assert_result(
            &[0xB2, 0x76, 0x34],
            &[MidiMessage::ControlChange(
                2.into(),
                0x76.into(),
                0x34.into(),
            )],
        );
    }

    #[test]
    fn should_parse_control_change_running_state() {
        MidiParser::new().assert_result(
            &[
                0xb3, 0x3C, 0x18, // First control change
                0x43, 0x01, // Second control change without status byte
            ],
            &[
                MidiMessage::ControlChange(3.into(), 0x3c.into(), 0x18.into()),
                MidiMessage::ControlChange(3.into(), 0x43.into(), 0x01.into()),
            ],
        );
    }

    #[test]
    fn should_parse_program_change() {
        MidiParser::new().assert_result(
            &[0xC9, 0x15],
            &[MidiMessage::ProgramChange(9.into(), 0x15.into())],
        );
    }

    #[test]
    fn should_parse_program_change_running_state() {
        MidiParser::new().assert_result(
            &[
                0xC3, 0x67, // First program change
                0x01, // Second program change without status byte
            ],
            &[
                MidiMessage::ProgramChange(3.into(), 0x67.into()),
                MidiMessage::ProgramChange(3.into(), 0x01.into()),
            ],
        );
    }

    #[test]
    fn should_parse_channel_pressure() {
        MidiParser::new().assert_result(
            &[0xDD, 0x37],
            &[MidiMessage::ChannelPressure(13.into(), 0x37.into())],
        );
    }

    #[test]
    fn should_parse_channel_pressure_running_state() {
        MidiParser::new().assert_result(
            &[
                0xD6, 0x77, // First channel pressure
                0x43, // Second channel pressure without status byte
            ],
            &[
                MidiMessage::ChannelPressure(6.into(), 0x77.into()),
                MidiMessage::ChannelPressure(6.into(), 0x43.into()),
            ],
        );
    }

    #[test]
    fn should_parse_pitchbend() {
        MidiParser::new().assert_result(
            &[0xE8, 0x14, 0x56],
            &[MidiMessage::PitchBendChange(8.into(), (0x56, 0x14).into())],
        );
    }

    #[test]
    fn should_parse_pitchbend_running_state() {
        MidiParser::new().assert_result(
            &[
                0xE3, 0x3C, 0x18, // First pitchbend
                0x43, 0x01, // Second pitchbend without status byte
            ],
            &[
                MidiMessage::PitchBendChange(3.into(), (0x18, 0x3c).into()),
                MidiMessage::PitchBendChange(3.into(), (0x01, 0x43).into()),
            ],
        );
    }

    #[test]
    fn should_parse_quarter_frame() {
        MidiParser::new().assert_result(&[0xf1, 0x7f], &[MidiMessage::QuarterFrame(0x7f.into())]);
    }

    #[test]
    fn should_handle_quarter_frame_running_state() {
        MidiParser::new().assert_result(
            &[
                0xf1, 0x7f, // Send quarter frame
                0x56, // Only send data of next quarter frame
            ],
            &[
                MidiMessage::QuarterFrame(0x7f.into()),
                MidiMessage::QuarterFrame(0x56.into()),
            ],
        );
    }

    #[test]
    fn should_parse_song_position_pointer() {
        MidiParser::new().assert_result(
            &[0xf2, 0x7f, 0x68],
            &[MidiMessage::SongPositionPointer((0x68, 0x7f).into())],
        );
    }

    #[test]
    fn should_handle_song_position_pointer_running_state() {
        MidiParser::new().assert_result(
            &[
                0xf2, 0x7f, 0x68, // Send song position pointer
                0x23, 0x7b, // Only send data of next song position pointer
            ],
            &[
                MidiMessage::SongPositionPointer((0x68, 0x7f).into()),
                MidiMessage::SongPositionPointer((0x7b, 0x23).into()),
            ],
        );
    }

    #[test]
    fn should_parse_song_select() {
        MidiParser::new().assert_result(&[0xf3, 0x3f], &[MidiMessage::SongSelect(0x3f.into())]);
    }

    #[test]
    fn should_handle_song_select_running_state() {
        MidiParser::new().assert_result(
            &[
                0xf3, 0x3f, // Send song select
                0x00, // Only send data for next song select
            ],
            &[
                MidiMessage::SongSelect(0x3f.into()),
                MidiMessage::SongSelect(0x00.into()),
            ],
        );
    }

    #[test]
    fn should_parse_tune_request() {
        MidiParser::new().assert_result(&[0xf6], &[MidiMessage::TuneRequest]);
    }

    #[test]
    fn should_interrupt_parsing_for_tune_request() {
        MidiParser::new().assert_result(
            &[
                0x92, 0x76, // start note_on message
                0xf6, // interrupt with tune request
                0x34, // finish note on, this should be ignored
            ],
            &[MidiMessage::TuneRequest],
        );
    }

    // #[test]
    // fn should_parse_end_exclusive() {
    //     MidiByteStreamParser::new().assert_result(&[0xf7], &[MidiMessage::EndOfExclusive]);
    // }

    // #[test]
    // fn should_interrupt_parsing_for_end_of_exclusive() {
    //     MidiByteStreamParser::new().assert_result(
    //         &[
    //             0x92, 0x76, // start note_on message
    //             0xf7, // interrupt with end of exclusive
    //             0x34, // finish note on, this should be ignored
    //         ],
    //         &[MidiMessage::EndOfExclusive],
    //     );
    // }

    #[test]
    fn should_interrupt_parsing_for_undefined_message() {
        MidiParser::new().assert_result(
            &[
                0x92, 0x76, // start note_on message
                0xf5, // interrupt with undefined message
                0x34, // finish note on, this should be ignored
            ],
            &[],
        );
    }

    #[test]
    fn should_parse_timingclock_message() {
        MidiParser::new().assert_result(&[0xf8], &[MidiMessage::TimingClock]);
    }

    #[test]
    fn should_parse_timingclock_message_as_realtime() {
        MidiParser::new().assert_result(
            &[
                0xD6, // Start channel pressure event
                0xf8, // interupt with midi timing clock
                0x77, // Finish channel pressure
            ],
            &[
                MidiMessage::TimingClock,
                MidiMessage::ChannelPressure(6.into(), 0x77.into()),
            ],
        );
    }

    #[test]
    fn should_parse_start_message() {
        MidiParser::new().assert_result(&[0xfa], &[MidiMessage::Start]);
    }

    #[test]
    fn should_parse_start_message_as_realtime() {
        MidiParser::new().assert_result(
            &[
                0xD6, // Start channel pressure event
                0xfa, // interupt with start
                0x77, // Finish channel pressure
            ],
            &[
                MidiMessage::Start,
                MidiMessage::ChannelPressure(6.into(), 0x77.into()),
            ],
        );
    }

    #[test]
    fn should_parse_continue_message() {
        MidiParser::new().assert_result(&[0xfb], &[MidiMessage::Continue]);
    }

    #[test]
    fn should_parse_continue_message_as_realtime() {
        MidiParser::new().assert_result(
            &[
                0xD6, // Start channel pressure event
                0xfb, // interupt with continue
                0x77, // Finish channel pressure
            ],
            &[
                MidiMessage::Continue,
                MidiMessage::ChannelPressure(6.into(), 0x77.into()),
            ],
        );
    }

    #[test]
    fn should_parse_stop_message() {
        MidiParser::new().assert_result(&[0xfc], &[MidiMessage::Stop]);
    }

    #[test]
    fn should_parse_stop_message_as_realtime() {
        MidiParser::new().assert_result(
            &[
                0xD6, // Start channel pressure event
                0xfc, // interupt with stop
                0x77, // Finish channel pressure
            ],
            &[
                MidiMessage::Stop,
                MidiMessage::ChannelPressure(6.into(), 0x77.into()),
            ],
        );
    }

    #[test]
    fn should_parse_activesensing_message() {
        MidiParser::new().assert_result(&[0xfe], &[MidiMessage::ActiveSensing]);
    }

    #[test]
    fn should_parse_activesensing_message_as_realtime() {
        MidiParser::new().assert_result(
            &[
                0xD6, // Start channel pressure event
                0xfe, // interupt with activesensing
                0x77, // Finish channel pressure
            ],
            &[
                MidiMessage::ActiveSensing,
                MidiMessage::ChannelPressure(6.into(), 0x77.into()),
            ],
        );
    }

    #[test]
    fn should_parse_reset_message() {
        MidiParser::new().assert_result(&[0xff], &[MidiMessage::Reset]);
    }

    #[test]
    fn should_parse_reset_message_as_realtime() {
        MidiParser::new().assert_result(
            &[
                0xD6, // Start channel pressure event
                0xff, // interupt with reset
                0x77, // Finish channel pressure
            ],
            &[
                MidiMessage::Reset,
                MidiMessage::ChannelPressure(6.into(), 0x77.into()),
            ],
        );
    }

    #[test]
    fn should_ignore_incomplete_messages() {
        MidiParser::new().assert_result(
            &[
                0x92, 0x1b, // Start note off message
                0x82, 0x76, 0x34, // continue with a complete note on message
            ],
            &[MidiMessage::NoteOff(2.into(), 0x76.into(), 0x34.into())],
        );
    }

    impl MidiParser {
        /// Test helper function, asserts if a slice of bytes parses to some set of midi events
        fn assert_result(&mut self, bytes: &[u8], expected_events: &[MidiMessage]) {
            let events: Vec<MidiMessage> =
                bytes.iter().filter_map(|byte| self.parse(*byte)).collect();

            assert_eq!(expected_events, events.as_slice());
        }
    }
}
