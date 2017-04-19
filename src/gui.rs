use gtk;
use gtk::prelude::*;
use std::cell::RefCell;
use portaudio as pa;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::RwLock;
use std::io;
use std::io::Write;
use std::thread;
use std::sync::mpsc::*;

struct RustyUi {
    dropdown: gtk::ComboBoxText,
    pitch_label: gtk::Label,
    pitch_error_indicator: gtk::DrawingArea,
    oscilloscope_chart: gtk::DrawingArea,
    freq_chart: gtk::DrawingArea,
    correlation_chart: gtk::DrawingArea,
    oscilloscope_toggle_button: gtk::Button,
    freq_toggle_button: gtk::Button,
    correlation_toggle_button: gtk::Button
}

struct ApplicationState {
    pa: pa::PortAudio,
    pa_stream: Option<pa::Stream<pa::NonBlocking, pa::Input<f32>>>,
    ui: RustyUi
}

struct CrossThreadState {
    pitch: String,
    error: f64,
    signal: Vec<f64>,
    freq_spectrum: Vec<::transforms::FrequencyBucket>,
    correlation: Vec<f64>
}

pub fn start_gui() -> Result<(), String> {
    let pa = try!(::audio::init().map_err(|e| e.to_string()));
    let microphones = try!(::audio::get_device_list(&pa).map_err(|e| e.to_string()));

    try!(gtk::init().map_err(|_| "Failed to initialize GTK."));

    let state = Rc::new(RefCell::new(ApplicationState {
        pa: pa,
        pa_stream: None,
        ui: create_window(microphones)
    }));

    let cross_thread_state = Arc::new(RwLock::new(CrossThreadState {
        pitch: String::new(),
        error: 0.0,
        signal: Vec::new(),
        freq_spectrum: Vec::new(),
        correlation: Vec::new()
    }));
    
    let (mic_sender, mic_receiver) = channel();

    connect_dropdown_choose_microphone(mic_sender, state.clone());
    
    start_processing_audio(mic_receiver, cross_thread_state.clone());
    setup_pitch_label_callbacks(state.clone(), cross_thread_state.clone());
    setup_pitch_error_indicator_callbacks(state.clone(), cross_thread_state.clone());
    setup_oscilloscope_drawing_area_callbacks(state.clone(), cross_thread_state.clone());
    setup_freq_drawing_area_callbacks(state.clone(), cross_thread_state.clone());
    setup_correlation_drawing_area_callbacks(state.clone(), cross_thread_state.clone());

    setup_chart_visibility_callbacks(state.clone());
    
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

    let vbox = gtk::Box::new(gtk::Orientation::Vertical, 5);
    window.add(&vbox);

    let hbox = gtk::Box::new(gtk::Orientation::Horizontal, 2);
    vbox.add(&hbox);
    let dropdown = gtk::ComboBoxText::new();
    dropdown.set_hexpand(true);
    set_dropdown_items(&dropdown, microphones);
    hbox.add(&dropdown);
    
    let oscilloscope_toggle_button = gtk::Button::new_with_label("Osc");
    hbox.add(&oscilloscope_toggle_button);
    let freq_toggle_button = gtk::Button::new_with_label("FFT");
    hbox.add(&freq_toggle_button);
    let correlation_toggle_button = gtk::Button::new_with_label("Corr");
    hbox.add(&correlation_toggle_button);

    let pitch_label = gtk::Label::new(None);
    vbox.add(&pitch_label);

    let pitch_error_indicator = gtk::DrawingArea::new();
    pitch_error_indicator.set_size_request(600, 50);
    vbox.add(&pitch_error_indicator);

    let oscilloscope_chart = gtk::DrawingArea::new();
    oscilloscope_chart.set_size_request(600, 250);
    oscilloscope_chart.set_vexpand(true);
    vbox.add(&oscilloscope_chart);
    
    let freq_chart = gtk::DrawingArea::new();
    freq_chart.set_size_request(600, 250);
    freq_chart.set_vexpand(true);
    vbox.add(&freq_chart);

    let correlation_chart = gtk::DrawingArea::new();
    correlation_chart.set_size_request(600, 250);
    correlation_chart.set_vexpand(true);
    vbox.add(&correlation_chart);

    window.show_all();
    
    RustyUi {
        dropdown: dropdown,
        pitch_label: pitch_label,
        pitch_error_indicator: pitch_error_indicator,
        oscilloscope_chart: oscilloscope_chart,
        freq_chart: freq_chart,
        correlation_chart: correlation_chart,
        oscilloscope_toggle_button: oscilloscope_toggle_button,
        freq_toggle_button: freq_toggle_button,
        correlation_toggle_button: correlation_toggle_button
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

fn start_processing_audio(mic_receiver: Receiver<Vec<f64>>, cross_thread_state: Arc<RwLock<CrossThreadState>>) {
    thread::spawn(move || {
        loop {
            let mut samples = None;
            loop {
                let next = mic_receiver.try_recv().ok();
                if next.is_none() {
                    break;
                }
                samples = next;
            }
            let samples = match samples {
                Some(samples) => samples,
                None => {continue;}
            };

            let signal = ::transforms::align_to_rising_edge(&samples);
            let frequency_domain = ::transforms::fft(&samples, 44100.0);
            let correlation = ::transforms::correlation(&samples);
            let fundamental = ::transforms::find_fundamental_frequency_correlation(&samples, 44100.0);
            let (pitch, error) = match fundamental {
                Some(fundamental) => (::transforms::hz_to_pitch(fundamental), ::transforms::hz_to_cents_error(fundamental)),
                None => ("".to_string(), 0.0)
            };

            match cross_thread_state.write() {
                Ok(mut state) => {
                    state.pitch = pitch;
                    state.signal = signal;
                    state.freq_spectrum = frequency_domain;
                    state.correlation = correlation;
                    state.error = error
                },
                Err(_) => {}
            };
        }
    });
}

fn setup_pitch_label_callbacks(state: Rc<RefCell<ApplicationState>>, cross_thread_state: Arc<RwLock<CrossThreadState>>) {
    gtk::timeout_add(100, move || {
        let ref pitch = cross_thread_state.read().unwrap().pitch;
        let ref ui = state.borrow().ui;
        ui.pitch_label.set_label(pitch.as_ref());
        ui.pitch_error_indicator.queue_draw();
        ui.oscilloscope_chart.queue_draw();
        ui.correlation_chart.queue_draw();
        ui.freq_chart.queue_draw();

        gtk::Continue(true)
    });
}

fn setup_pitch_error_indicator_callbacks(state: Rc<RefCell<ApplicationState>>, cross_thread_state: Arc<RwLock<CrossThreadState>>) {
    let outer_state = state.clone();
    let ref canvas = outer_state.borrow().ui.pitch_error_indicator;
    canvas.connect_draw(move |ref canvas, ref context| {
        let error = cross_thread_state.read().unwrap().error;
        let width = canvas.get_allocated_width() as f64;
        let midpoint = width / 2.0;
        let height = canvas.get_allocated_height() as f64;

        //flat on the left
        context.set_source_rgb(0.0, 0.0, if error < 0.0 {-error/50.0} else {0.0});
        context.rectangle(0.0, 0.0, midpoint, height);
        context.fill();

        //sharp on the right
        context.set_source_rgb(if error > 0.0 {error/50.0} else {0.0}, 0.0, 0.0);
        context.rectangle(midpoint, 0.0, width, height);
        context.fill();
        
        gtk::Inhibit(false)
    });
}

fn setup_oscilloscope_drawing_area_callbacks(state: Rc<RefCell<ApplicationState>>, cross_thread_state: Arc<RwLock<CrossThreadState>>) {
    let outer_state = state.clone();
    let ref canvas = outer_state.borrow().ui.oscilloscope_chart;
    canvas.connect_draw(move |ref canvas, ref context| {
        let ref signal = cross_thread_state.read().unwrap().signal;
        let width = canvas.get_allocated_width() as f64;
        let len = 512.0; //Set as a constant so signal won't change size based on zero point.
        let height = canvas.get_allocated_height() as f64;
        let mid_height = height / 2.0;
        let max = signal.iter().map(|x| x.abs()).fold(0.0, |max, x| if max > x { max } else { x });

        context.new_path();
        context.move_to(0.0, mid_height);

        for (i, intensity) in signal.iter().enumerate() {
            let x = i as f64 * width / len;
            let y = mid_height - (intensity * mid_height / max);
            context.line_to(x, y);
        }

        context.stroke();
        
        gtk::Inhibit(false)
    });
}

fn setup_freq_drawing_area_callbacks(state: Rc<RefCell<ApplicationState>>, cross_thread_state: Arc<RwLock<CrossThreadState>>) {
    let outer_state = state.clone();
    let ref canvas = outer_state.borrow().ui.freq_chart;
    canvas.connect_draw(move |ref canvas, ref context| {
        let ref spectrum = cross_thread_state.read().unwrap().freq_spectrum;
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

fn setup_correlation_drawing_area_callbacks(state: Rc<RefCell<ApplicationState>>, cross_thread_state: Arc<RwLock<CrossThreadState>>) {
    let outer_state = state.clone();
    let ref canvas = outer_state.borrow().ui.correlation_chart;
    canvas.connect_draw(move |ref canvas, ref context| {
        let ref correlation = cross_thread_state.read().unwrap().correlation;
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

fn setup_chart_visibility_callbacks(state: Rc<RefCell<ApplicationState>>) {
    let outer_state = state.clone();
    let ref oscilloscope_toggle_button = outer_state.borrow().ui.oscilloscope_toggle_button;
    let ref freq_toggle_button = outer_state.borrow().ui.freq_toggle_button;
    let ref correlation_toggle_button = outer_state.borrow().ui.correlation_toggle_button;

    let oscilloscope_state = state.clone();
    oscilloscope_toggle_button.connect_clicked(move |_| {
        let ref chart = oscilloscope_state.borrow().ui.oscilloscope_chart;
        chart.set_visible(!chart.get_visible());
    });
    
    let freq_state = state.clone();
    freq_toggle_button.connect_clicked(move |_| {
        let ref chart = freq_state.borrow().ui.freq_chart;
        chart.set_visible(!chart.get_visible());
    });

    let correlation_state = state.clone();
    correlation_toggle_button.connect_clicked(move |_| {
        let ref chart = correlation_state.borrow().ui.correlation_chart;
        chart.set_visible(!chart.get_visible());
    });
}
