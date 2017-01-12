use gtk;
use gtk::prelude::*;
use std::cell::RefCell;
use portaudio as pa;
use std::rc::Rc;
use std::io;
use std::io::Write;
use std::thread;
use std::sync::mpsc::*;

struct RustyUi {
    window: gtk::Window,
    dropdown: gtk::ComboBoxText,
    pitch_label: gtk::Label,
    freq_chart: gtk::DrawingArea
}

struct ApplicationState {
    pa: pa::PortAudio,
    pa_stream: Option<pa::Stream<pa::NonBlocking, pa::Input<f32>>>
}

pub fn start_gui() -> Result<(), String> {
    let pa = try!(::audio::init().map_err(|e| e.to_string()));
    let microphones = try!(::audio::get_device_list(&pa).map_err(|e| e.to_string()));

    let (mic_sender, mic_receiver) = channel();
    let state = Rc::new(RefCell::new(ApplicationState {
        pa: pa,
        pa_stream: None
    }));
    
    try!(gtk::init().map_err(|_| "Failed to initialize GTK."));

    let ui = create_window(microphones);

    connect_dropdown_choose_microphone(&ui.dropdown, mic_sender, state.clone());

    let (pitch_sender, pitch_receiver) = channel();
    let (freq_sender, freq_receiver) = channel();
    
    start_processing_audio(mic_receiver, pitch_sender, freq_sender);
    
    setup_pitch_label_callbacks(ui.pitch_label, pitch_receiver);
    setup_drawing_area_callbacks(ui.freq_chart, freq_receiver);

    gtk::main();
    Ok(())
}

fn create_window(microphones: Vec<(u32, String)>) -> RustyUi {
    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    window.set_title("Rusty Microphone");
    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });

    let layout_box = gtk::Box::new(gtk::Orientation::Vertical, 5);
    window.add(&layout_box);

    let dropdown = gtk::ComboBoxText::new();
    set_dropdown_items(&dropdown, microphones);
    layout_box.add(&dropdown);

    let pitch_label = gtk::Label::new(None);
    layout_box.add(&pitch_label);

    let freq_chart = gtk::DrawingArea::new();
    freq_chart.set_size_request(600, 400);
    layout_box.add(&freq_chart);

    window.show_all();
    
    RustyUi {
        window: window,
        dropdown: dropdown,
        pitch_label: pitch_label,
        freq_chart: freq_chart
    }
}

fn set_dropdown_items(dropdown: &gtk::ComboBoxText, microphones: Vec<(u32, String)>) {
    for (index, name) in microphones {
        dropdown.append(Some(format!("{}", index).as_ref()), name.as_ref());
    }
}

fn connect_dropdown_choose_microphone(dropdown: &gtk::ComboBoxText, mic_sender: Sender<Vec<f64>>, state: Rc<RefCell<ApplicationState>>) {
    dropdown.connect_changed(move |dropdown: &gtk::ComboBoxText| {
        match state.borrow_mut().pa_stream {
            Some(ref mut stream) => {stream.stop().ok();},
            _ => {}
        }
        let selected_mic = match dropdown.get_active_id().and_then(|id| id.parse().ok()) {
            Some(mic) => mic,
            None => {return;}
        };
        let stream = ::audio::start_listening(&state.borrow().pa, selected_mic, mic_sender.clone()).ok();
        if stream.is_none() {
            writeln!(io::stderr(), "Failed to open audio channel").ok();
        }
        state.borrow_mut().pa_stream = stream;
    });
}

fn start_processing_audio(mic_receiver: Receiver<Vec<f64>>, pitch_sender: Sender<String>, freq_sender: Sender<Vec<::transforms::FrequencyBucket>>) {
    thread::spawn(move || {
        for samples in mic_receiver {
            let frequency_domain = ::transforms::fft(samples, 44100.0);
            freq_sender.send(frequency_domain.clone());
            let fundamental = ::transforms::find_fundamental_frequency(&frequency_domain);
            let pitch = match fundamental {
                Some(fundamental) => ::transforms::hz_to_pitch(fundamental),
                None => "".to_string()
            };
            pitch_sender.send(pitch).ok();
        }
    });
}

fn setup_pitch_label_callbacks(pitch_label: gtk::Label, pitch_receiver: Receiver<String>) {
    gtk::timeout_add(100, move || {
        let mut pitch = None;
        loop {
            let next = pitch_receiver.try_recv().ok();
            if next.is_none() {
                break;
            }
            pitch = next;
        }
        match pitch {
            Some(pitch) => {pitch_label.set_label(pitch.as_ref());},
            None => {}
        };
        gtk::Continue(true)
    });
}

fn setup_drawing_area_callbacks(canvas: gtk::DrawingArea, freq_receiver: Receiver<Vec<::transforms::FrequencyBucket>>) {
    canvas.connect_draw(move |ref canvas, ref context| {
        let mut last_signal = Vec::new();
        loop {
            let next = freq_receiver.try_recv().ok();
            if next.is_none() {
                break;
            }
            last_signal = next.unwrap();
        }

        context.new_path();
        context.move_to(0.0, 0.0);
        context.line_to(canvas.get_allocated_width() as f64, canvas.get_allocated_height() as f64);
        context.stroke();
        
        gtk::Inhibit(false)
    });
}
