use gtk;
use gtk::prelude::*;
use std::cell::Cell;

const GUI_XML: &'static str = r#"
<interface>
  <object class="GtkWindow" id="window">
    <property name="title">Rusty Microphone</property>
    <child>
      <object class="GtkComboBoxText" id="dropdown">
      </object>
    </child>
  </object>
</interface>
"#;

pub fn start_gui() -> Result<(), String> {
    let pa = try!(::audio::init().map_err(|e| e.to_string()));
    let microphones = try!(::audio::get_device_list(&pa).map_err(|e| e.to_string()));
    let selected_mic: Cell<Option<u32>> = Cell::new(None);
    
    try!(gtk::init().map_err(|_| "Failed to initialize GTK."));

    let gtk_builder = try!(create_window(microphones));
    
    let dropdown: gtk::ComboBoxText = try!(
        gtk_builder.get_object("dropdown")
                   .ok_or("GUI does not contain an object with id 'dropdown'")
    );
    dropdown.connect_changed(move |dropdown: &gtk::ComboBoxText| {
        selected_mic.set(dropdown.get_active_id().and_then(|id| id.parse().ok()));        
        println!("{}", selected_mic.get().unwrap());
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
