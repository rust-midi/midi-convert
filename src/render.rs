use midi_types::{status::*, MidiMessage};

/// This trait abstracts the transport mechanism for the MidiRenderer. An instance of a type that implements this trait can be used by the MidiRenderer to write midi-messages
pub trait MidiTransport {
    type Error;

    /// Write a message as series of bytes to the midi transport layer
    ///
    /// For compatibility this should always be used to write one whole midi-message with a maximum of 3 bytes
    fn write(&mut self, bytes: &[u8]) -> Result<(), Self::Error>;
}

/// The MidiRenderer takes MIDI messages and writes them to the underlying transport, the boolean const generic RUNNING_STATUS enables or disables rendering running status for midi messages
#[derive(Debug)]
pub struct MidiRenderer<T, const RUNNING_STATUS: bool = true> {
    transport: T,
    running_status: Option<u8>,
}

impl<T: MidiTransport, const RUNNING_STATUS: bool> MidiRenderer<T, RUNNING_STATUS> {
    pub fn new(transport: T) -> Self {
        Self {
            transport,
            running_status: None,
        }
    }

    pub fn release(self) -> T {
        self.transport
    }

    pub fn render(&mut self, message: &MidiMessage) -> Result<(), T::Error> {
        match *message {
            // Channel voice messages
            MidiMessage::NoteOn(channel, note, velocity) => {
                self.write_channel_msg(&[
                    NOTE_ON + Into::<u8>::into(channel),
                    note.into(),
                    velocity.into(),
                ])?;
            }
            MidiMessage::NoteOff(channel, note, velocity) => {
                self.write_channel_msg(&[
                    NOTE_OFF + Into::<u8>::into(channel),
                    note.into(),
                    velocity.into(),
                ])?;
            }
            MidiMessage::KeyPressure(channel, note, value) => {
                self.write_channel_msg(&[
                    KEY_PRESSURE + Into::<u8>::into(channel),
                    note.into(),
                    value.into(),
                ])?;
            }
            MidiMessage::ControlChange(channel, control, value) => {
                self.write_channel_msg(&[
                    CONTROL_CHANGE + Into::<u8>::into(channel),
                    control.into(),
                    value.into(),
                ])?;
            }
            MidiMessage::ProgramChange(channel, program) => {
                self.write_channel_msg(&[
                    PROGRAM_CHANGE + Into::<u8>::into(channel),
                    program.into(),
                ])?;
            }
            MidiMessage::ChannelPressure(channel, value) => {
                self.write_channel_msg(&[
                    CHANNEL_PRESSURE + Into::<u8>::into(channel),
                    value.into(),
                ])?;
            }
            MidiMessage::PitchBendChange(channel, value) => {
                let (msb, lsb) = value.into();
                self.write_channel_msg(&[PITCH_BEND_CHANGE + Into::<u8>::into(channel), lsb, msb])?;
            }

            // System common messages
            MidiMessage::QuarterFrame(value) => {
                self.write_sys_common_msg(&[QUARTER_FRAME, value.into()])?;
            }
            MidiMessage::SongPositionPointer(value) => {
                let (msb, lsb) = value.into();
                self.write_sys_common_msg(&[SONG_POSITION_POINTER, lsb, msb])?;
            }
            MidiMessage::SongSelect(value) => {
                self.write_sys_common_msg(&[SONG_SELECT, value.into()])?;
            }
            MidiMessage::TuneRequest => {
                self.write_sys_common_msg(&[TUNE_REQUEST])?;
            }

            // System real time messages
            MidiMessage::TimingClock => self.transport.write(&[TIMING_CLOCK])?,
            MidiMessage::Start => self.transport.write(&[START])?,
            MidiMessage::Continue => self.transport.write(&[CONTINUE])?,
            MidiMessage::Stop => self.transport.write(&[STOP])?,
            MidiMessage::ActiveSensing => self.transport.write(&[ACTIVE_SENSING])?,
            MidiMessage::Reset => self.transport.write(&[RESET])?,
        }
        Ok(())
    }

    /// Write a channel voice or channel mode messages, these messages optionally use running status to
    /// skip sending the status byte
    fn write_channel_msg(&mut self, data: &[u8]) -> Result<(), T::Error> {
        let status = data[0];
        if RUNNING_STATUS && self.running_status == Some(status) {
            // If the last command written had the same status/channel, the MIDI protocol allows us to
            // omit sending the status byte again.
            self.transport.write(&data[1..])?;
        } else {
            self.transport.write(data)?;

            if RUNNING_STATUS {
                // Store running state so the next message can use it
                self.running_status = Some(status);
            }
        }

        Ok(())
    }

    /// Write a System common message, these messages do not use running status but do reset it
    fn write_sys_common_msg(&mut self, data: &[u8]) -> Result<(), T::Error> {
        self.transport.write(data)?;

        if RUNNING_STATUS {
            self.running_status = None;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use midi_types::Note;

    extern crate std;
    use std::{string::String, vec::Vec};

    // Test if all midi messages are rendered correctly

    #[test]
    fn should_render_note_on() {
        assert_eq!(
            render::<true>(&[MidiMessage::NoteOn(2.into(), Note::C3, 0x34.into())]),
            &[0x92, 0x3c, 0x34]
        );
    }

    #[test]
    fn should_render_note_off() {
        assert_eq!(
            render::<true>(&[MidiMessage::NoteOff(2.into(), Note::C3, 0x34.into())]),
            &[0x82, 0x3c, 0x34],
        );
    }

    #[test]
    fn should_render_keypressure() {
        assert_eq!(
            render::<true>(&[MidiMessage::KeyPressure(2.into(), 0x76.into(), 0x34.into())]),
            &[0xA2, 0x76, 0x34],
        );
    }

    #[test]
    fn should_render_control_change() {
        assert_eq!(
            render::<true>(&[MidiMessage::ControlChange(
                2.into(),
                0x76.into(),
                0x34.into(),
            )]),
            &[0xB2, 0x76, 0x34],
        );
    }

    #[test]
    fn should_render_program_change() {
        assert_eq!(
            render::<true>(&[MidiMessage::ProgramChange(2.into(), 0x76.into())]),
            &[0xC2, 0x76],
        );
    }

    #[test]
    fn should_render_channel_pressure() {
        assert_eq!(
            render::<true>(&[MidiMessage::ChannelPressure(2.into(), 0x76.into())]),
            &[0xD2, 0x76],
        );
    }

    #[test]
    fn should_render_pitchbend() {
        assert_eq!(
            render::<true>(&[MidiMessage::PitchBendChange(8.into(), (0x56, 0x14).into(),)]),
            &[0xE8, 0x14, 0x56],
        );
    }

    #[test]
    fn should_render_quarter_frame() {
        assert_eq!(
            render::<true>(&[MidiMessage::QuarterFrame(0x76.into())]),
            &[0xF1, 0x76]
        );
    }

    #[test]
    fn should_render_song_position_pointer() {
        assert_eq!(
            render::<true>(&[MidiMessage::SongPositionPointer((0x68, 0x7f).into())]),
            &[0xf2, 0x7f, 0x68],
        );
    }

    #[test]
    fn should_render_song_select() {
        assert_eq!(
            render::<true>(&[MidiMessage::SongSelect(0x76.into())]),
            &[0xF3, 0x76]
        );
    }

    #[test]
    fn should_render_tune_request() {
        assert_eq!(render::<true>(&[MidiMessage::TuneRequest]), &[0xF6]);
    }

    #[test]
    fn should_render_timing_clock() {
        assert_eq!(render::<true>(&[MidiMessage::TimingClock]), &[0xF8]);
    }

    #[test]
    fn should_render_start() {
        assert_eq!(render::<true>(&[MidiMessage::Start]), &[0xFA]);
    }

    #[test]
    fn should_render_continue() {
        assert_eq!(render::<true>(&[MidiMessage::Continue]), &[0xFB]);
    }

    #[test]
    fn should_render_stop() {
        assert_eq!(render::<true>(&[MidiMessage::Stop]), &[0xFC]);
    }

    #[test]
    fn should_render_active_sensing() {
        assert_eq!(render::<true>(&[MidiMessage::ActiveSensing]), &[0xFE]);
    }

    #[test]
    fn should_render_reset() {
        assert_eq!(render::<true>(&[MidiMessage::Reset]), &[0xFF]);
    }

    // Test running status

    #[test]
    fn should_skip_repeated_status_with_running_status_on() {
        assert_eq!(
            render::<true>(&[
                MidiMessage::NoteOn(2.into(), Note::D4, 0x34.into()),
                MidiMessage::NoteOn(2.into(), Note::G6, 0x65.into()),
            ]),
            &[0x92, 0x4a, 0x34, 0x67, 0x65],
        );
    }

    #[test]
    fn should_not_skip_repeated_status_with_running_status_off() {
        assert_eq!(
            render::<false>(&[
                MidiMessage::NoteOn(2.into(), Note::D4, 0x34.into()),
                MidiMessage::NoteOff(2.into(), Note::G6, 0x65.into()),
            ]),
            &[0x92, 0x4a, 0x34, 0x82, 0x67, 0x65],
        );
    }

    #[test]
    fn should_not_skip_status_when_channel_changes() {
        assert_eq!(
            render::<true>(&[
                MidiMessage::NoteOn(2.into(), Note::D4, 0x34.into()),
                MidiMessage::NoteOn(3.into(), Note::G6, 0x65.into()),
            ]),
            &[0x92, 0x4a, 0x34, 0x93, 0x67, 0x65],
        );
    }

    #[test]
    fn should_not_skip_status_for_different_message() {
        assert_eq!(
            render::<true>(&[
                MidiMessage::NoteOn(2.into(), Note::D4, 0x34.into()),
                MidiMessage::NoteOff(2.into(), Note::G6, 0x65.into()),
            ]),
            &[0x92, 0x4a, 0x34, 0x82, 0x67, 0x65],
        );
    }

    // Some test helpers

    #[derive(Debug, Default, Clone)]
    struct MockTransport {
        buffer: Vec<u8>,
    }

    impl MidiTransport for MockTransport {
        type Error = String;

        fn write(&mut self, bytes: &[u8]) -> Result<(), Self::Error> {
            // Make sure all messages fit in a USB midi packet
            assert!(bytes.len() <= 3, "Too many bytes in one message");
            bytes.iter().for_each(|value| self.buffer.push(*value));
            Ok(())
        }
    }

    fn render<const RUNNING_STATUS: bool>(messages: &[MidiMessage]) -> Vec<u8> {
        let mut renderer: MidiRenderer<MockTransport, RUNNING_STATUS> =
            MidiRenderer::new(MockTransport::default());
        for message in messages {
            renderer.render(message).expect("Error rendering message");
        }

        renderer.transport.buffer
    }
}
