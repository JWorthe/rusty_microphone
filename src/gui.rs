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
    freq_chart: gtk::DrawingArea,
    correlation_chart: gtk::DrawingArea
}

struct ApplicationState {
    pa: pa::PortAudio,
    pa_stream: Option<pa::Stream<pa::NonBlocking, pa::Input<f32>>>,
    freq_spectrum: Vec<::transforms::FrequencyBucket>,
    correlation: Vec<f64>,
    ui: RustyUi
}

pub fn start_gui() -> Result<(), String> {
    let pa = try!(::audio::init().map_err(|e| e.to_string()));
    let microphones = try!(::audio::get_device_list(&pa).map_err(|e| e.to_string()));

    try!(gtk::init().map_err(|_| "Failed to initialize GTK."));

    let state = Rc::new(RefCell::new(ApplicationState {
        pa: pa,
        pa_stream: None,
        freq_spectrum: Vec::new(),
        correlation: Vec::new(),
        ui: create_window(microphones)
    }));

    //let ui = create_window(microphones);
    
    let (mic_sender, mic_receiver) = channel();
    let (pitch_sender, pitch_receiver) = channel();
    let (freq_sender, freq_receiver) = channel();
    let (correlation_sender, correlation_receiver) = channel();

    connect_dropdown_choose_microphone(mic_sender, state.clone());
    
    start_processing_audio(mic_receiver, pitch_sender, freq_sender, correlation_sender);
    setup_pitch_label_callbacks(pitch_receiver, state.clone());
    setup_freq_drawing_area_callbacks(freq_receiver, state.clone());
    setup_correlation_drawing_area_callbacks(correlation_receiver, state.clone());

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
    freq_chart.set_no_show_all(true);

    let correlation_chart = gtk::DrawingArea::new();
    correlation_chart.set_size_request(600, 400);
    layout_box.add(&correlation_chart);
    correlation_chart.set_no_show_all(true);

    window.show_all();
    
    RustyUi {
        window: window,
        dropdown: dropdown,
        pitch_label: pitch_label,
        freq_chart: freq_chart,
        correlation_chart: correlation_chart
    }
}

fn set_dropdown_items(dropdown: &gtk::ComboBoxText, microphones: Vec<(u32, String)>) {
    for (index, name) in microphones {
        dropdown.append(Some(format!("{}", index).as_ref()), name.as_ref());
    }
}

fn connect_dropdown_choose_microphone(mic_sender: Sender<Vec<f64>>, state: Rc<RefCell<ApplicationState>>) {
    let outer_state = state.clone();
    let ref dropdown = outer_state.borrow().ui.dropdown;
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

fn start_processing_audio(mic_receiver: Receiver<Vec<f64>>, pitch_sender: Sender<String>, freq_sender: Sender<Vec<::transforms::FrequencyBucket>>, correlation_sender: Sender<Vec<f64>>) {
    thread::spawn(move || {
        for samples in mic_receiver {
            //let frequency_domain = ::transforms::fft(&samples, 44100.0);
            //freq_sender.send(frequency_domain).ok();

            //let correlation = ::transforms::correlation(&samples);
            //correlation_sender.send(correlation).ok();
            
            let fundamental = ::transforms::find_fundamental_frequency_correlation(&samples, 44100.0);
            let pitch = match fundamental {
                Some(fundamental) => ::transforms::hz_to_pitch(fundamental),
                None => "".to_string()
            };
            pitch_sender.send(pitch).ok();
        }
    });
}

fn setup_pitch_label_callbacks(pitch_receiver: Receiver<String>, state: Rc<RefCell<ApplicationState>>) {
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
            Some(pitch) => {state.borrow().ui.pitch_label.set_label(pitch.as_ref());},
            None => {}
        };
        state.borrow().ui.freq_chart.queue_draw();
        gtk::Continue(true)
    });
}

fn setup_freq_drawing_area_callbacks(spectrum_receiver: Receiver<Vec<::transforms::FrequencyBucket>>, state: Rc<RefCell<ApplicationState>>) {
    setup_frequency_spectrum_callback(spectrum_receiver, state.clone());

    let outer_state = state.clone();
    let ref canvas = outer_state.borrow().ui.freq_chart;
    canvas.connect_draw(move |ref canvas, ref context| {
        let ref spectrum = state.borrow().freq_spectrum;
        let width = canvas.get_allocated_width() as f64;
        let height = canvas.get_allocated_height() as f64;
        let max = spectrum.iter().map(|x| x.intensity).fold(0.0, |max, x| if max > x { max } else { x });
        let len = spectrum.len() as f64;
        
        context.new_path();
        context.move_to(0.0, height);
        
        for (i, bucket) in spectrum.iter().enumerate() {
            let x = i as f64 * width / len;
            let y = height - (bucket.intensity * height / max);
            context.line_to(x, y);
        }
        
        context.stroke();
        
        gtk::Inhibit(false)
    });
}

fn setup_frequency_spectrum_callback(spectrum_receiver: Receiver<Vec<::transforms::FrequencyBucket>>, state: Rc<RefCell<ApplicationState>>) {
    gtk::timeout_add(100, move || {
        let mut spectrum = None;
        loop {
            let next = spectrum_receiver.try_recv().ok();
            if next.is_none() {
                break;
            }
            spectrum = next;
        }
        match spectrum {
            Some(spectrum) => {
                state.borrow_mut().freq_spectrum = spectrum;
                state.borrow().ui.freq_chart.queue_draw();
            },
            None => {}
        };
        gtk::Continue(true)
    });
}

fn setup_correlation_drawing_area_callbacks(correlation_receiver: Receiver<Vec<f64>>, state: Rc<RefCell<ApplicationState>>) {
    setup_correlation_callback(correlation_receiver, state.clone());

    let outer_state = state.clone();
    let ref canvas = outer_state.borrow().ui.correlation_chart;
    canvas.connect_draw(move |ref canvas, ref context| {
        let ref correlation = state.borrow().correlation;
        if correlation.len() == 0 {
            return gtk::Inhibit(false);
        }
        
        let width = canvas.get_allocated_width() as f64;
        let height = canvas.get_allocated_height() as f64;
        let max = correlation[0];
        let len = correlation.len() as f64;
        
        context.new_path();
        context.move_to(0.0, height);
        
        for (i, val) in correlation.iter().enumerate() {
            let x = i as f64 * width / len;
            let y = height - (val * height / max);
            context.line_to(x, y);
        }
        
        context.stroke();
        
        gtk::Inhibit(false)
    });
}

fn setup_correlation_callback(correlation_receiver: Receiver<Vec<f64>>, state: Rc<RefCell<ApplicationState>>) {
    gtk::timeout_add(100, move || {
        let mut correlation = None;
        loop {
            let next = correlation_receiver.try_recv().ok();
            if next.is_none() {
                break;
            }
            correlation = next;
        }
        match correlation {
            Some(correlation) => {
                state.borrow_mut().correlation = correlation;
                state.borrow().ui.correlation_chart.queue_draw();
            },
            None => {}
        };
        gtk::Continue(true)
    });
}
