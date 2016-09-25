use gtk;
use gtk::prelude::*;

pub fn start_gui() -> Result<(), String> {
    try!(gtk::init().map_err(|_| "Failed to initialize GTK."));
        
    // Create the main window.
    let window = gtk::Window::new(gtk::WindowType::Toplevel);
    window.set_title("Musician Training");

    let audio_devices = try!(::audio::get_device_list().map_err(|e| e.to_string()));
    let dropdown = gtk::ComboBoxText::new();
    for (index, name) in audio_devices {
        dropdown.append(Some(format!("{}", index).as_ref()), name.as_ref());
    }
    window.add(&dropdown);
    window.set_default_size(300, 300);

    window.show_all();

    window.connect_delete_event(|_, _| {
        gtk::main_quit();
        gtk::prelude::Inhibit(false)
    });

    gtk::main();
    Ok(())
}
