use gtk;
use gtk::prelude::*;
use std::cell::RefCell;
use portaudio as pa;
use std::rc::Rc;
use std::io;
use std::io::Write;
use std::thread;
use std::sync::mpsc::*;

const GUI_XML: &'static str = r#"
<interface>
  <object class="GtkWindow" id="window">
    <property name="title">Rusty Microphone</property>
    <child>
      <object class="GtkVBox">
        <child>
          <object class="GtkComboBoxText" id="dropdown">
          </object>
        </child>
        <child>
          <object class="GtkLabel" id="pitch-label">
            <property name="label">Hello world</property>
          </object>
        </child>
      </object>
    </child>
  </object>
</interface>
"#;

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

    let gtk_builder = try!(create_window(microphones));

    connect_dropdown_choose_microphone(&gtk_builder, mic_sender, state.clone());

    let (pitch_sender, pitch_receiver) = channel();
    
    start_processing_audio(mic_receiver, pitch_sender);
        
    let pitch_label: gtk::Label = gtk_builder.get_object("pitch-label").expect("GUI does not contain an object with id 'pitch-label'");
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

    gtk::main();
    Ok(())
}

fn create_window(microphones: Vec<(u32, String)>) -> Result<gtk::Builder, String> {
    let gtk_builder = gtk::Builder::new_from_string(GUI_XML);
    let window: gtk::Window = try!(
        gtk_builder.get_object("window")
                   .ok_or("GUI does not contain an object with id 'window'")
    );
    window.set_default_size(300, 300);
    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        Inhibit(false)
    });
    window.show_all();

    let dropdown: gtk::ComboBoxText = try!(
        gtk_builder.get_object("dropdown")
                   .ok_or("GUI does not contain an object with id 'dropdown'")
    );
    set_dropdown_items(&dropdown, microphones);

    Ok(gtk_builder)
}

fn set_dropdown_items(dropdown: &gtk::ComboBoxText, microphones: Vec<(u32, String)>) {
    for (index, name) in microphones {
        dropdown.append(Some(format!("{}", index).as_ref()), name.as_ref());
    }
}

fn connect_dropdown_choose_microphone(gtk_builder: &gtk::Builder, mic_sender: Sender<Vec<f64>>, state: Rc<RefCell<ApplicationState>>) {
    let dropdown: gtk::ComboBoxText = gtk_builder.get_object("dropdown").expect("GUI does not contain an object with id 'dropdown'");
    dropdown.connect_changed(move |dropdown: &gtk::ComboBoxText| {
        match state.borrow_mut().pa_stream {
            Some(ref mut stream) => {stream.stop().ok();},
            _ => {}
        }
        let selected_mic = dropdown.get_active_id().and_then(|id| id.parse().ok()).expect("Dropdown did not change to a valid value");
        let stream = ::audio::start_listening(&state.borrow().pa, selected_mic, mic_sender.clone()).ok();
        if stream.is_none() {
            writeln!(io::stderr(), "Failed to open audio channel").ok();
        }
        state.borrow_mut().pa_stream = stream;
    });
}

fn start_processing_audio(mic_receiver: Receiver<Vec<f64>>, pitch_sender: Sender<String>) {
    thread::spawn(move || {
        for samples in mic_receiver {
            let frequency_domain = ::transforms::fft(samples, 44100.0);
            let fundamental = ::transforms::find_fundamental_frequency(&frequency_domain);
            let pitch = match fundamental {
                Some(fundamental) => ::transforms::hz_to_pitch(fundamental),
                None => "".to_string()
            };
            pitch_sender.send(pitch).ok();
        }
    });
}
