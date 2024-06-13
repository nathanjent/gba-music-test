#![no_std]
#![no_main]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]
#![cfg_attr(test, test_runner(agb::test_runner::test_runner))]
#![feature(slice_pattern)]
extern crate alloc;

use agb::display::object::ObjectTextRender;
use agb::display::object::PaletteVram;
use agb::display::object::Size;
use agb::display::object::TextAlignment;
use agb::display::palette16::Palette16;
use agb::display::Font;
use agb::display::WIDTH;
use agb::include_font;
use agb::input::Button;
use agb::input::ButtonController;
use agb::interrupt::VBlank;
use agb::sound::dmg::EnvelopeSettings;
use agb::sound::dmg::SoundDirection;
use agb::sound::dmg::SweepSettings;
use core::fmt::Write;

pub const FONT: Font = include_font!("fonts/yoster.ttf", 12);

pub enum Note {
    C,
    CSharp,
    D,
    DSharp,
    E,
    F,
    FSharp,
    G,
    GSharp,
    A,
    ASharp,
    B,
}

pub enum Octave {
    D2,
    D1,
    U0,
    U1,
    U2,
    U3,
    U4,
    U5,
    U6,
    U7,
    U8,
}

impl Octave {
    fn value(&self) -> i8 {
        match *self {
            Octave::D2 => -2,
            Octave::D1 => -1,
            Octave::U0 => 0,
            Octave::U1 => 1,
            Octave::U2 => 2,
            Octave::U3 => 3,
            Octave::U4 => 4,
            Octave::U5 => 5,
            Octave::U6 => 6,
            Octave::U7 => 7,
            Octave::U8 => 8,
        }
    }
}

impl Note {
    fn value(&self) -> u32 {
        match *self {
            Note::C => 8013,
            Note::CSharp => 7566,
            Note::D => 7144,
            Note::DSharp => 6742,
            Note::E => 6362,
            Note::F => 6005,
            Note::FSharp => 5666,
            Note::G => 5346,
            Note::GSharp => 5048,
            Note::A => 4766,
            Note::ASharp => 4499,
            Note::B => 4246,
        }
    }
}

pub fn sound_rate(note: &Note, octave: &Octave) -> u16 {
    2048 - (note.value() >> (4 + octave.value())) as u16
}

pub const C1: u16 = 262;
pub const C_SHARP1: u16 = 277;
pub const D1: u16 = 293;
pub const D_SHARP1: u16 = 311;
pub const E1: u16 = 330;
pub const F1: u16 = 349;
pub const F_SHARP1: u16 = 370;
pub const G1: u16 = 392;
pub const G_SHARP1: u16 = 415;
pub const A1: u16 = 440;
pub const A_SHARP1: u16 = 466;
pub const B1: u16 = 494;
pub const C2: u16 = 523;

pub fn entry(mut gba: agb::Gba) -> ! {
    let vblank = VBlank::get();
    let (mut unmanaged, mut sprite_loader) = gba.display.object.get_unmanaged();

    let ch1 = gba.sound.channel1();
    let ch2 = gba.sound.channel2();
    let noise = gba.sound.noise();
    gba.sound.enable();

    let envelope_settings_ch1 = EnvelopeSettings::new(7, SoundDirection::Decrease, 12);
    let sweep_settings_ch1 = SweepSettings::default();
    let envelope_settings_ch2 = EnvelopeSettings::new(2, SoundDirection::Decrease, 1);
    let envelope_settings_noise = EnvelopeSettings::new(2, SoundDirection::Decrease, 1);

    let button_ctrl = ButtonController::new();

    let mut palette = [0x0; 16];
    palette[1] = 0xFF_FF;
    let palette = Palette16::new(palette);
    let palette = PaletteVram::new(&palette).unwrap();
    let mut writer = ObjectTextRender::new(&FONT, Size::S16x16, palette);
    let _ = writeln!(writer, "Play some music!");
    writer.layout((WIDTH, 40), TextAlignment::Left, 2);

    // Note, octave, hold_time
    let song = [
        (Note::C, Octave::U2, 24),
        (Note::C, Octave::U2, 24),
        (Note::D, Octave::U2, 12),
        (Note::E, Octave::U2, 12),
        (Note::F, Octave::U2, 12),
        (Note::C, Octave::U1, 24),
        (Note::C, Octave::U1, 24),
        (Note::F, Octave::U1, 12),
        (Note::E, Octave::U1, 12),
        (Note::D, Octave::U1, 12),
    ];

    let mut tic = 0;
    let mut song_cursor = 0;
    let mut next_note_tic = 0;

    loop {
        writer.next_letter_group();
        writer.update((0, 0));
        vblank.wait_for_vblank();

        let (ref note, ref octave, length) = song[song_cursor];
        if tic >= next_note_tic {
            next_note_tic = tic + length;
            ch1.play_sound(
                sound_rate(&note, &octave),
                None,
                &sweep_settings_ch1,
                &envelope_settings_ch1,
                agb::sound::dmg::DutyCycle::Half,
            );
            song_cursor += 1;
            if song_cursor >= song.len() {
                song_cursor = 0;
            }
        }

        let oam = &mut unmanaged.iter();

        if button_ctrl.is_pressed(Button::A) {}
        if button_ctrl.is_released(Button::A) {}

        writer.commit(oam);
        tic += 1;
    }
}
