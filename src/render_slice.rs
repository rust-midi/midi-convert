//! Render message to a byte buffer

use midi_types::{MidiMessage, status::*};

/// Trait for rendering a MidiMessage into a byte slice.
pub trait MidiRenderSlice: Sized {
    /// Render to a slice.
    ///
    /// # Panics
    ///
    /// Panics if the slice length is less than 3.
    fn render_slice(&self, buf: &mut [u8]) -> usize;
}

//helper to render 3 byte messages
fn chan3byte<T0: Into<u8> + Copy, T1: Into<u8> + Copy, C: Into<u8> + Copy>(
    buf: &mut [u8],
    status: u8,
    chan: &C,
    d0: &T0,
    d1: &T1,
) -> usize {
    let chan: u8 = (*chan).into();
    let status = status | chan;
    for (o, i) in buf.iter_mut().zip(&[status, (*d0).into(), (*d1).into()]) {
        *o = *i;
    }
    3
}

//helper to render 2 byte messages
fn chan2byte<T0: Into<u8> + Copy, C: Into<u8> + Copy>(
    buf: &mut [u8],
    status: u8,
    chan: &C,
    d0: &T0,
) -> usize {
    let chan: u8 = (*chan).into();
    let status = status | chan;
    for (o, i) in buf.iter_mut().zip(&[status, (*d0).into()]) {
        *o = *i;
    }
    2
}

//helper to render 1 byte messages
fn chan1byte(buf: &mut [u8], status: u8) -> usize {
    buf[0] = status;
    1
}

impl MidiRenderSlice for MidiMessage {
    /// Render into a raw byte buffer, return the number of bytes rendered
    fn render_slice(&self, buf: &mut [u8]) -> usize {
        assert!(buf.len() >= 3);
        match self {
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
}

#[cfg(test)]
mod test {
    use {
        super::*,
        crate::test::{TEST_1BYTE, TEST_2BYTE, TEST_3BYTE},
    };

    #[test]
    #[should_panic]
    fn render_1_0_panic() {
        let mut buf: [u8; 0] = [];
        (*TEST_1BYTE)[0].render_slice(&mut buf);
    }

    #[test]
    #[should_panic]
    fn render_1_1_panic() {
        let mut buf: [u8; 1] = [0; 1];
        (*TEST_1BYTE)[0].render_slice(&mut buf);
    }

    #[test]
    #[should_panic]
    fn render_1_2_panic() {
        let mut buf: [u8; 2] = [0; 2];
        (*TEST_1BYTE)[0].render_slice(&mut buf);
    }

    #[test]
    #[should_panic]
    fn render_2_2_panic() {
        let mut buf: [u8; 2] = [0; 2];
        (*TEST_2BYTE)[0].render_slice(&mut buf);
    }

    #[test]
    #[should_panic]
    fn render_3_1_panic() {
        let mut buf: [u8; 2] = [0; 2];
        (*TEST_3BYTE)[0].render_slice(&mut buf);
    }

    #[test]
    fn render_ok() {
        let mut buf3 = [0, 0, 0];
        let mut buf100 = [0; 100];
        for v in (*TEST_1BYTE).iter() {
            assert_eq!(1, v.render_slice(&mut buf3), "{:?}", v);
            assert_eq!(1, v.render_slice(&mut buf100), "{:?}", v);
        }

        for v in (*TEST_2BYTE).iter() {
            assert_eq!(2, v.render_slice(&mut buf3), "{:?}", v);
            assert_eq!(2, v.render_slice(&mut buf100), "{:?}", v);
        }

        for v in (*TEST_3BYTE).iter() {
            assert_eq!(3, v.render_slice(&mut buf3), "{:?}", v);
            assert_eq!(3, v.render_slice(&mut buf100), "{:?}", v);
        }
    }
}
