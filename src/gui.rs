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

const FPS: u32 = 30;

struct RustyUi {
    dropdown: gtk::ComboBoxText,
    pitch_label: gtk::Label,
    pitch_error_indicator: gtk::DrawingArea,
    oscilloscope_chart: gtk::DrawingArea,
    correlation_chart: gtk::DrawingArea,
    oscilloscope_toggle_button: gtk::Button,
    correlation_toggle_button: gtk::Button
}

struct ApplicationState {
    pa: pa::PortAudio,
    pa_stream: Option<pa::Stream<pa::NonBlocking, pa::Input<f32>>>,
    ui: RustyUi
}

struct CrossThreadState {
    fundamental_frequency: Option<f32>,
    pitch: String,
    error: Option<f32>,
    signal: Vec<f32>,
    correlation: Vec<f32>
}

pub fn start_gui() -> Result<(), String> {
    let pa = try!(::audio::init().map_err(|e| e.to_string()));
    let microphones = try!(::audio::get_device_list(&pa).map_err(|e| e.to_string()));
    let default_microphone = try!(::audio::get_default_device(&pa).map_err(|e| e.to_string()));
    
    try!(gtk::init().map_err(|_| "Failed to initialize GTK."));

    let state = Rc::new(RefCell::new(ApplicationState {
        pa: pa,
        pa_stream: None,
        ui: create_window(microphones, default_microphone)
    }));

    let cross_thread_state = Arc::new(RwLock::new(CrossThreadState {
        fundamental_frequency: None,
        pitch: String::new(),
        error: None,
        signal: Vec::new(),
        correlation: Vec::new()
    }));
    
    let (mic_sender, mic_receiver) = channel();

    connect_dropdown_choose_microphone(mic_sender, state.clone());
    
    start_processing_audio(mic_receiver, cross_thread_state.clone());
    setup_pitch_label_callbacks(state.clone(), cross_thread_state.clone());
    setup_pitch_error_indicator_callbacks(state.clone(), cross_thread_state.clone());
    setup_oscilloscope_drawing_area_callbacks(state.clone(), cross_thread_state.clone());
    setup_correlation_drawing_area_callbacks(state.clone(), cross_thread_state.clone());

    setup_chart_visibility_callbacks(state.clone());
    
    gtk::main();
    Ok(())
}

fn create_window(microphones: Vec<(u32, String)>, default_microphone: u32) -> RustyUi {
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
    set_dropdown_items(&dropdown, microphones, default_microphone);
    hbox.add(&dropdown);
    
    let oscilloscope_toggle_button = gtk::Button::new_with_label("Osc");
    hbox.add(&oscilloscope_toggle_button);
    let correlation_toggle_button = gtk::Button::new_with_label("Corr");
    hbox.add(&correlation_toggle_button);

    let pitch_label = gtk::Label::new(None);
    vbox.add(&pitch_label);

    let pitch_error_indicator = gtk::DrawingArea::new();
    pitch_error_indicator.set_size_request(600, 70);
    vbox.add(&pitch_error_indicator);

    let oscilloscope_chart = gtk::DrawingArea::new();
    oscilloscope_chart.set_size_request(600, 250);
    oscilloscope_chart.set_vexpand(true);
    vbox.add(&oscilloscope_chart);
    
    let correlation_chart = gtk::DrawingArea::new();
    correlation_chart.set_size_request(600, 250);
    correlation_chart.set_vexpand(true);
    vbox.add(&correlation_chart);

    window.show_all();
    
    // correlation chart is only really useful for debugging, so it
    // makes sense to have it default to being hidden
    correlation_chart.set_visible(false);
    
    RustyUi {
        dropdown: dropdown,
        pitch_label: pitch_label,
        pitch_error_indicator: pitch_error_indicator,
        oscilloscope_chart: oscilloscope_chart,
        correlation_chart: correlation_chart,
        oscilloscope_toggle_button: oscilloscope_toggle_button,
        correlation_toggle_button: correlation_toggle_button
    }
}

fn set_dropdown_items(dropdown: &gtk::ComboBoxText, microphones: Vec<(u32, String)>, default_mic: u32) {
    for (index, name) in microphones {
        dropdown.append(Some(format!("{}", index).as_ref()), name.as_ref());
    }
    dropdown.set_active_id(Some(format!("{}", default_mic).as_ref()));
}

fn connect_dropdown_choose_microphone(mic_sender: Sender<Vec<f32>>, state: Rc<RefCell<ApplicationState>>) {
    let dropdown = state.borrow().ui.dropdown.clone();
    start_listening_current_dropdown_value(&dropdown, mic_sender.clone(), state.clone());
    dropdown.connect_changed(move |dropdown: &gtk::ComboBoxText| {
        start_listening_current_dropdown_value(&dropdown, mic_sender.clone(), state.clone())
    });
}

fn start_listening_current_dropdown_value(dropdown: &gtk::ComboBoxText, mic_sender: Sender<Vec<f32>>, state: Rc<RefCell<ApplicationState>>) {
    match state.borrow_mut().pa_stream {
        Some(ref mut stream) => {stream.stop().ok();},
        _ => {}
    }
    let selected_mic = match dropdown.get_active_id().and_then(|id| id.parse().ok()) {
        Some(mic) => mic,
        None => {return;}
    };
    let stream = ::audio::start_listening(&state.borrow().pa, selected_mic, mic_sender).ok();
    if stream.is_none() {
        writeln!(io::stderr(), "Failed to open audio channel").ok();
    }
    state.borrow_mut().pa_stream = stream;
}

fn start_processing_audio(mic_receiver: Receiver<Vec<f32>>, cross_thread_state: Arc<RwLock<CrossThreadState>>) {
    thread::spawn(move || {
        while let Ok(samples) = mic_receiver.recv() {
            let signal = ::transforms::align_to_rising_edge(&samples);
            let correlation = ::transforms::correlation(&samples);
            let fundamental = ::transforms::find_fundamental_frequency_correlation(&samples, ::audio::SAMPLE_RATE);
            let pitch = match fundamental {
                Some(fundamental) => ::transforms::hz_to_pitch(fundamental),
                None => String::new()
            };
            let error = fundamental.map(::transforms::hz_to_cents_error);

            match cross_thread_state.write() {
                Ok(mut state) => {
                    state.fundamental_frequency = fundamental;
                    state.pitch = pitch;
                    state.signal = signal;
                    state.correlation = correlation;
                    state.error = error;
                },
                Err(err) => {
                    println!("Error updating cross thread state: {}", err);
                }
            };
        }
    });
}

fn setup_pitch_label_callbacks(state: Rc<RefCell<ApplicationState>>, cross_thread_state: Arc<RwLock<CrossThreadState>>) {
    gtk::timeout_add(1000/FPS, move || {
        let ref ui = state.borrow().ui;
        if let Ok(cross_thread_state) = cross_thread_state.read() {
            let ref pitch = cross_thread_state.pitch;
            ui.pitch_label.set_label(pitch.as_ref());
            ui.pitch_error_indicator.queue_draw();
            ui.oscilloscope_chart.queue_draw();
            ui.correlation_chart.queue_draw();
        }

        gtk::Continue(true)
    });
}

fn setup_pitch_error_indicator_callbacks(state: Rc<RefCell<ApplicationState>>, cross_thread_state: Arc<RwLock<CrossThreadState>>) {
    let outer_state = state.clone();
    let ref canvas = outer_state.borrow().ui.pitch_error_indicator;
    canvas.connect_draw(move |ref canvas, ref context| {
        let width = canvas.get_allocated_width() as f64;
        let midpoint = width / 2.0;

        let line_indicator_height = 20.0;
        let color_indicator_height = canvas.get_allocated_height() as f64 - line_indicator_height;

        match cross_thread_state.read().map(|state| state.error) {
            Ok(Some(error)) =>  {
                let error_line_x = midpoint + error as f64 * midpoint / 50.0;
                context.new_path();
                context.move_to(error_line_x, 0.0);
                context.line_to(error_line_x, line_indicator_height);
                context.stroke();
                
                
                //flat on the left
                context.set_source_rgb(0.0, 0.0, if error < 0.0 {-error as f64/50.0} else {0.0});
                context.rectangle(0.0, line_indicator_height, midpoint, color_indicator_height+line_indicator_height);
                context.fill();

                //sharp on the right
                context.set_source_rgb(if error > 0.0 {error as f64/50.0} else {0.0}, 0.0, 0.0);
                context.rectangle(midpoint, line_indicator_height, width, color_indicator_height+line_indicator_height);
                context.fill();
            },
            _ => {
                context.set_source_rgb(0.0, 0.0, 0.0);
                context.rectangle(0.0, line_indicator_height, width, color_indicator_height+line_indicator_height);
                context.fill();
            }
        }
        
        gtk::Inhibit(false)
    });
}

fn setup_oscilloscope_drawing_area_callbacks(state: Rc<RefCell<ApplicationState>>, cross_thread_state: Arc<RwLock<CrossThreadState>>) {
    let outer_state = state.clone();
    let ref canvas = outer_state.borrow().ui.oscilloscope_chart;
    canvas.connect_draw(move |ref canvas, ref context| {
        if let Ok(cross_thread_state) = cross_thread_state.read() {
            let ref signal = cross_thread_state.signal;
            let width = canvas.get_allocated_width() as f64;
            let len = 512.0; //Set as a constant so signal won't change size based on zero point.
            let height = canvas.get_allocated_height() as f64;
            let mid_height = height / 2.0;
            let max = 1.0;

            context.new_path();
            context.move_to(0.0, mid_height);

            for (i, &intensity) in signal.iter().enumerate() {
                let x = i as f64 * width / len;
                let y = mid_height - (intensity as f64 * mid_height / max);
                context.line_to(x, y);
            }

            context.stroke();
        }
        
        gtk::Inhibit(false)
    });
}

fn setup_correlation_drawing_area_callbacks(state: Rc<RefCell<ApplicationState>>, cross_thread_state: Arc<RwLock<CrossThreadState>>) {
    let outer_state = state.clone();
    let ref canvas = outer_state.borrow().ui.correlation_chart;
    canvas.connect_draw(move |ref canvas, ref context| {
        let width = canvas.get_allocated_width() as f64;
        let height = canvas.get_allocated_height() as f64;
        
        //draw zero
        context.new_path();
        context.move_to(0.0, height/2.0);
        context.line_to(width, height/2.0);
        context.stroke();

        if let Ok(cross_thread_state) = cross_thread_state.read() {
            let ref correlation = cross_thread_state.correlation;
            let len = correlation.len() as f64;
            let max = match correlation.first() {
                Some(&c) => c as f64,
                None => 1.0
            };
            
            context.new_path();
            context.move_to(0.0, height);
            for (i, &val) in correlation.iter().enumerate() {
                let x = i as f64 * width / len;
                let y = height/2.0 - (val as f64 * height / max / 2.0);
                context.line_to(x, y);
            }        
            context.stroke();

            //draw the fundamental
            if let Some(fundamental) = cross_thread_state.fundamental_frequency {
                context.new_path();
                let fundamental_x = ::audio::SAMPLE_RATE as f64 / fundamental as f64 * width / len;
                context.move_to(fundamental_x, 0.0);
                context.line_to(fundamental_x, height);
                context.stroke();
            }
        }
        
        gtk::Inhibit(false)
    });
}

fn setup_chart_visibility_callbacks(state: Rc<RefCell<ApplicationState>>) {
    let outer_state = state.clone();
    let ref oscilloscope_toggle_button = outer_state.borrow().ui.oscilloscope_toggle_button;
    let ref correlation_toggle_button = outer_state.borrow().ui.correlation_toggle_button;

    let oscilloscope_state = state.clone();
    oscilloscope_toggle_button.connect_clicked(move |_| {
        let ref chart = oscilloscope_state.borrow().ui.oscilloscope_chart;
        chart.set_visible(!chart.get_visible());
    });

    let correlation_state = state.clone();
    correlation_toggle_button.connect_clicked(move |_| {
        let ref chart = correlation_state.borrow().ui.correlation_chart;
        chart.set_visible(!chart.get_visible());
    });
}
