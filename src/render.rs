//! Render message to a byte buffer

use midi_types::{status::*, MidiMessage};

#[derive(Debug, PartialEq, Clone)]
/// Errors rendering
pub enum RenderError {
    ///Input buffer wasn't long enough to render message
    BufferTooShort,
}

//helper to render 3 byte messages
fn chan3byte<T0: Into<u8> + Copy, T1: Into<u8> + Copy, C: Into<u8> + Copy>(
    buf: &mut [u8],
    status: u8,
    chan: &C,
    d0: &T0,
    d1: &T1,
) -> Result<usize, RenderError> {
    if buf.len() >= 3 {
        let chan: u8 = (*chan).into();
        let status = status | chan;
        for (o, i) in buf.iter_mut().zip(&[status, (*d0).into(), (*d1).into()]) {
            *o = *i;
        }
        Ok(3)
    } else {
        Err(RenderError::BufferTooShort)
    }
}

//helper to render 2 byte messages
fn chan2byte<T0: Into<u8> + Copy, C: Into<u8> + Copy>(
    buf: &mut [u8],
    status: u8,
    chan: &C,
    d0: &T0,
) -> Result<usize, RenderError> {
    if buf.len() >= 2 {
        let chan: u8 = (*chan).into();
        let status = status | chan;
        for (o, i) in buf.iter_mut().zip(&[status, (*d0).into()]) {
            *o = *i;
        }
        Ok(2)
    } else {
        Err(RenderError::BufferTooShort)
    }
}

//helper to render 1 byte messages
fn chan1byte(buf: &mut [u8], status: u8) -> Result<usize, RenderError> {
    if buf.len() >= 1 {
        buf[0] = status;
        Ok(1)
    } else {
        Err(RenderError::BufferTooShort)
    }
}

/// Render into a raw byte buffer, return the number of bytes rendered
pub fn render(msg: &MidiMessage, buf: &mut [u8]) -> Result<usize, RenderError> {
    match msg {
        MidiMessage::NoteOff(c, n, v) => chan3byte(buf, NOTE_OFF, c, n, v),
        MidiMessage::NoteOn(c, n, v) => chan3byte(buf, NOTE_ON, c, n, v),
        MidiMessage::KeyPressure(c, n, v) => chan3byte(buf, KEY_PRESSURE, c, n, v),
        MidiMessage::ControlChange(c, n, v) => chan3byte(buf, CONTROL_CHANGE, c, n, v),
        MidiMessage::PitchBendChange(c, v) => {
            let (v0, v1): (u8, u8) = (*v).into();
            chan3byte(buf, PITCH_BEND_CHANGE, c, &v0, &v1)
        }
        MidiMessage::SongPositionPointer(v) => {
            let (v0, v1): (u8, u8) = (*v).into();
            chan3byte(buf, SONG_POSITION_POINTER, &0, &v0, &v1)
        }
        MidiMessage::ProgramChange(c, p) => chan2byte(buf, PROGRAM_CHANGE, c, p),
        MidiMessage::ChannelPressure(c, p) => chan2byte(buf, CHANNEL_PRESSURE, c, p),
        MidiMessage::QuarterFrame(q) => chan2byte(buf, QUARTER_FRAME, &0, q),
        MidiMessage::SongSelect(s) => chan2byte(buf, SONG_SELECT, &0, s),
        MidiMessage::TuneRequest => chan1byte(buf, TUNE_REQUEST),
        MidiMessage::TimingClock => chan1byte(buf, TIMING_CLOCK),
        MidiMessage::Start => chan1byte(buf, START),
        MidiMessage::Continue => chan1byte(buf, CONTINUE),
        MidiMessage::Stop => chan1byte(buf, STOP),
        MidiMessage::ActiveSensing => chan1byte(buf, ACTIVE_SENSING),
        MidiMessage::Reset => chan1byte(buf, RESET),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test::{TEST_1BYTE, TEST_2BYTE, TEST_3BYTE};

    #[test]
    fn render_err() {
        let mut buf0: [u8; 0] = [];
        let mut buf1 = [0];
        let mut buf2 = [0, 0];
        for v in (*TEST_1BYTE).iter() {
            assert_eq!(
                Err(RenderError::BufferTooShort),
                render(&v, &mut buf0),
                "{:?}",
                v
            );
        }

        for v in (*TEST_2BYTE).iter() {
            assert_eq!(
                Err(RenderError::BufferTooShort),
                render(&v, &mut buf0),
                "{:?}",
                v
            );
            assert_eq!(
                Err(RenderError::BufferTooShort),
                render(&v, &mut buf1),
                "{:?}",
                v
            );
        }

        for v in (*TEST_3BYTE).iter() {
            assert_eq!(
                Err(RenderError::BufferTooShort),
                render(&v, &mut buf0),
                "{:?}",
                v
            );
            assert_eq!(
                Err(RenderError::BufferTooShort),
                render(&v, &mut buf1),
                "{:?}",
                v
            );
            assert_eq!(
                Err(RenderError::BufferTooShort),
                render(&v, &mut buf2),
                "{:?}",
                v
            );
        }
    }

    #[test]
    fn render_ok() {
        let mut buf1 = [0];
        let mut buf2 = [0, 0];
        let mut buf3 = [0, 0, 0];
        let mut buf100 = [0; 100];
        for v in (*TEST_1BYTE).iter() {
            assert_eq!(Ok(1), render(&v, &mut buf1), "{:?}", v);
            assert_eq!(Ok(1), render(&v, &mut buf2), "{:?}", v);
            assert_eq!(Ok(1), render(&v, &mut buf100), "{:?}", v);
        }

        for v in (*TEST_2BYTE).iter() {
            assert_eq!(Ok(2), render(&v, &mut buf2), "{:?}", v);
            assert_eq!(Ok(2), render(&v, &mut buf100), "{:?}", v);
        }

        for v in (*TEST_3BYTE).iter() {
            assert_eq!(Ok(3), render(&v, &mut buf3), "{:?}", v);
            assert_eq!(Ok(3), render(&v, &mut buf100), "{:?}", v);
        }
    }
}
