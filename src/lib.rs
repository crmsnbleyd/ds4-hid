pub mod runner{
    extern crate hidapi;
    use hidapi::HidApi;
    use std::fmt;
    use std::io::Write;
    const REPORT_LEN: usize = 64;
    pub fn run() -> Result<(), String> {
	let mgr = HidApi::new().unwrap();
	let dpad_dirs = [DPad::Up, DPad::Down,
	                 DPad::UpLeft, DPad::UpRight,
			 DPad::DownLeft,DPad::DownRight,
	                 DPad::Right, DPad::Left];
	let devices = mgr.device_list();

	let (vid, pid) = (0x054c, 0x09cc); // enter your own device's product id

	let _info = devices.into_iter()
	    .find(|info| info.vendor_id() == vid && info.product_id() == pid)
	    .ok_or_else(|| format!("Failed to find a HID with a Vendor ID of {:#X} and a Product ID of {:#X}. Is your controller plugged in?", vid, pid))?;

	let dev = mgr.open(vid, pid).map_err(|_msg|
					     String::from("Failed to open the DS4 device. Have you setup `udev` rules? (if not, temporarily run `sudo ./target/debug/main`)"))?;

	let mut buf = [0u8; REPORT_LEN];

	let stdout = std::io::stdout();
	let mut lock = stdout.lock();

	let mut report: Report;
	lock.write(b"Welcome to the Controller to stdout driver.\nPress Cross to continue\n").expect("Writing to stdout failed");
	loop {
	    let _res = dev.read(&mut buf[..]);
	    report = Report::from_bytes(buf);
	    if (report.data[5]>>4) & 2 != 0 {break;}
	}
	loop {
	    let mut dpad_pressed = false;
	    let _res = dev.read(&mut buf[..]);
	    report = Report::from_bytes(buf);
	    
	    lock.write(format!("Square:   {}\n", report.is_button_pressed(Button::Square))
		       .as_bytes()).expect("Writing to stdout failed");
	    lock.write(format!("Cross:    {}\n", report.is_button_pressed(Button::Cross))
		       .as_bytes()).expect("Writing to stdout failed");
	    lock.write(format!("Circle:   {}\n", report.is_button_pressed(Button::Circle))
		       .as_bytes()).expect("Writing to stdout failed");
	    lock.write(format!("Triangle: {}\n", report.is_button_pressed(Button::Triangle))
		       .as_bytes()).expect("Writing to stdout failed");

	    for dir in dpad_dirs.iter(){
		if report.is_dpad_pressed(*dir) {
		    lock.write(format!("DPad Direction: {}\n", dir).as_bytes())
			.expect("Writing to stdout failed");
		    dpad_pressed = true;
		} 
	    }

	    if !dpad_pressed {lock.write(b"DPad Direction: None\n")
			      .expect("Writing to stdout failed");}

	    if report.is_option_pressed() {
		lock.write(b"Option button pressed. Quitting.\n")
		    .expect("Writing to stdout failed");
		break;
	    }
	    lock.write(b"\n").expect("Writing to stdout failed");
	    lock.flush().expect("Writing to stdout failed");
	}
	Ok(())
    }

    struct Report {
	data: Vec<u8>,
    }

    impl Report {
	pub fn from_bytes(data: [u8; 64]) -> Self {
	    Self {
		data: data.to_vec(),
	    }
	}

	pub fn is_button_pressed(&self, button: Button) -> String {
	    debug_assert_eq!(REPORT_LEN, self.data.len());

	    let nybble = self.data[5] >> 4;
	    if match button {
		Button::Square   => nybble & 1 != 0,
		Button::Cross    => nybble & 2 != 0,
		Button::Circle   => nybble & 4 != 0,
		Button::Triangle => nybble & 8 != 0,
	    } {"Pressed".to_owned()}
	    else {
		"Not Pressed".to_owned()
	    }
	}
	pub fn is_option_pressed(&self) -> bool {
	    self.data[6] & 32 != 0 // 5th bit of byte[6]
	}
	pub fn is_dpad_pressed(&self, dpad: DPad) -> bool {
	    debug_assert_eq!(REPORT_LEN, self.data.len());

	    let nybble = self.data[5] & 0x0f;
	    match dpad {
		// NOTE: 0x8 represents "no dpad pressed".
		DPad::Up        => nybble == 0b0000,
		DPad::UpRight   => nybble == 0b0001,
		DPad::UpLeft    => nybble == 0b0111,
		DPad::Right     => nybble == 0b0010,
		DPad::Down      => nybble == 0b0100,
		DPad::DownRight => nybble == 0b0011,
		DPad::DownLeft  => nybble == 0b0101,
		DPad::Left      => nybble == 0b0110,
	    }
	}
    }

    enum Button {
	Square,
	Cross,
	Circle,
	Triangle,
    }
    #[derive(Debug, Copy, Clone)]
    enum DPad {
	Up,
	UpRight,
	UpLeft,
	Right,
	Down,
	DownRight,
	DownLeft,
	Left,
    }
    impl fmt::Display for DPad{
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result{
	    write!(f, "{:?}", self)
	}
    }    
}
