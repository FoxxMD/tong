use std::io::{stdout, Write};
use input::{Libinput, LibinputInterface};
use libc::{O_RDONLY, O_RDWR, O_WRONLY};
use std::fs::{File, OpenOptions};
use std::os::unix::{fs::OpenOptionsExt, io::OwnedFd};
use std::path::Path;
use nix::poll::{poll, PollFlags, PollFd};
use std::os::fd::AsRawFd;
use libc::{signal, SIGPIPE, SIG_DFL};
use input::event::Event::{Keyboard, Pointer};
use input::event::keyboard::KeyboardEvent::{Key};
use input::event::keyboard::KeyState;
use input::event::keyboard::KeyboardEventTrait;
use input::event::pointer::PointerEvent::{MotionAbsolute,Motion};


struct Interface;

impl LibinputInterface for Interface {
    fn open_restricted(&mut self, path: &Path, flags: i32) -> Result<OwnedFd, i32> {
        OpenOptions::new()
            .custom_flags(flags)
            .read((flags & O_RDONLY != 0) | (flags & O_RDWR != 0))
            .write((flags & O_WRONLY != 0) | (flags & O_RDWR != 0))
            .open(path)
            .map(|file| file.into())
            .map_err(|err| err.raw_os_error().unwrap())
    }
    fn close_restricted(&mut self, fd: OwnedFd) {
        drop(File::from(fd));
    }
}


fn main() {

    // https://gitlab.com/somini/inpulse-to-talk/-/blob/897c5cbc98d34b2d81aa9387bc0912dc36abce91/src/bin/inpulse-daemon.rs
    let mut input = Libinput::new_with_udev(Interface);
    input.udev_assign_seat("seat0").unwrap();

    unsafe {
        signal(SIGPIPE, SIG_DFL);
    }

    let pollfd = PollFd::new(input.as_raw_fd(), PollFlags::POLLIN);

    println!("Listening");
    let mut stdout = stdout();
    while poll(&mut [pollfd], -1).is_ok() {
        input.dispatch().unwrap();
        for event in &mut input {
            if let Keyboard(Key(eventk)) = &event {
                match eventk.key_state() {
                    KeyState::Pressed => println!("Pressed key {:?}", eventk.key()),
                    _ => ()
                }
                } else if let Pointer(MotionAbsolute(eventm)) = &event {
                    println!("Mouse abs movement: {:?}", eventm);
                }  else if let Pointer(Motion(eventm)) = &event {
                    println!("Mouse pos: {}x {}y", eventm.dx(), eventm.dy());
                }
            // Make sure to flush stdoutasd
            // not sure this is necessary...
            stdout.flush().unwrap();
            }
        }
}